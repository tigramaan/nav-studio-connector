#!/usr/bin/env bash
set -euo pipefail

deb_path=""
appimage_path=""
while (($#)); do
  case "$1" in
    --deb) deb_path="${2:-}"; shift 2 ;;
    --appimage) appimage_path="${2:-}"; shift 2 ;;
    *) echo "Unknown or incomplete argument: $1" >&2; exit 2 ;;
  esac
done

if [[ ${EUID:-$(id -u)} -ne 0 ]]; then
  echo "Ubuntu package HIL must run as root in an isolated disposable environment" >&2
  exit 5
fi
if [[ -z "$deb_path" || ! -f "$deb_path" ]]; then
  echo "--deb must reference an existing package" >&2
  exit 2
fi
if [[ -n "$appimage_path" && ! -f "$appimage_path" ]]; then
  echo "--appimage must reference an existing package" >&2
  exit 2
fi
deb_path="$(readlink -f -- "$deb_path")"
if [[ -n "$appimage_path" ]]; then
  appimage_path="$(readlink -f -- "$appimage_path")"
fi
if ! systemd-detect-virt --container >/dev/null 2>&1 && [[ ${CONNECTOR_HIL_ALLOW_EPHEMERAL_HOST:-0} != 1 ]]; then
  echo "Refusing system trust mutation outside a container or explicitly authorized ephemeral host" >&2
  exit 5
fi

work="$(mktemp -d /tmp/nav-studio-connector-hil.XXXXXX)"
server_pid=""
package_name="$(dpkg-deb -f "$deb_path" Package)"
trust_target=""
cleanup() {
  if [[ -n "$server_pid" ]]; then kill "$server_pid" 2>/dev/null || true; fi
  if [[ -n "$trust_target" && "$trust_target" == /usr/local/share/ca-certificates/umec-nav-studio-*.crt ]]; then
    rm -f -- "$trust_target"
    update-ca-certificates >/dev/null 2>&1 || true
  fi
  apt-get remove -y "$package_name" >/dev/null 2>&1 || true
  case "$work" in /tmp/nav-studio-connector-hil.*) rm -rf -- "$work" ;; esac
}
trap cleanup EXIT

apt-get install -y "$deb_path" >/dev/null
binary="$(dpkg -L "$package_name" | awk '/\/usr\/bin\// { print; exit }')"
if [[ -z "$binary" || ! -x "$binary" ]]; then
  echo "Installed connector executable was not found" >&2
  exit 1
fi

describe_json="$($binary agent describe --json)"
python3 -c 'import json,sys; value=json.load(sys.stdin); assert value["ok"] is True and value["operation"] == "agent.describe"' <<<"$describe_json"

if [[ -n "$appimage_path" ]]; then
  chmod +x "$appimage_path"
  appimage_json="$(APPIMAGE_EXTRACT_AND_RUN=1 "$appimage_path" agent describe --json)"
  python3 -c 'import json,sys; value=json.load(sys.stdin); assert value["ok"] is True' <<<"$appimage_json"
fi

mkdir -p /run/user/0
chmod 0700 /run/user/0
export XDG_RUNTIME_DIR=/run/user/0
set +e
timeout 5s dbus-run-session -- xvfb-run -a "$binary" >"$work/gui.stdout" 2>"$work/gui.stderr"
gui_status=$?
set -e
if [[ $gui_status -ne 124 ]]; then
  echo "Installed GUI did not stay alive for the smoke interval (exit $gui_status)" >&2
  sed -n '1,80p' "$work/gui.stderr" >&2
  exit 1
fi

openssl req -x509 -newkey rsa:2048 -sha256 -nodes -days 1 \
  -subj "/CN=UMEC Nav Connector Disposable HIL" \
  -addext "subjectAltName=DNS:localhost,IP:127.0.0.1" \
  -addext "basicConstraints=critical,CA:TRUE" \
  -addext "keyUsage=critical,digitalSignature,keyCertSign" \
  -keyout "$work/server.key" -out "$work/server.crt" >/dev/null 2>&1
fingerprint="$(openssl x509 -in "$work/server.crt" -outform DER | sha256sum | awk '{print toupper($1)}')"
port=18443
python3 - "$work/server.crt" "$work/server.key" "$port" <<'PY' &
import http.server
import ssl
import sys

certificate, key, port = sys.argv[1], sys.argv[2], int(sys.argv[3])

class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        body = b'{"status":"ok"}'
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_args):
        return

server = http.server.ThreadingHTTPServer(("127.0.0.1", port), Handler)
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain(certificate, key)
server.socket = context.wrap_socket(server.socket, server_side=True)
server.serve_forever()
PY
server_pid=$!
for _attempt in $(seq 1 20); do
  if openssl s_client -connect "127.0.0.1:$port" -servername localhost </dev/null >/dev/null 2>&1; then break; fi
  sleep 0.2
done

if curl --fail --silent --show-error --max-time 3 "https://localhost:$port/" >/dev/null 2>&1; then
  echo "Disposable certificate was unexpectedly trusted before installation" >&2
  exit 1
fi

install_json="$($binary trust install --url "https://localhost:$port/" --expected-fingerprint "$fingerprint" --json)"
python3 -c 'import json,sys; value=json.load(sys.stdin); assert value["ok"] is True; assert value["data"]["health_state"] == "healthy"' <<<"$install_json"
trust_target="/usr/local/share/ca-certificates/umec-nav-studio-${fingerprint:0:16}.crt"
[[ -f "$trust_target" ]]
curl --fail --silent --show-error --max-time 3 "https://localhost:$port/" >/dev/null

remove_json="$($binary trust remove --fingerprint "$fingerprint" --json)"
python3 -c 'import json,sys; value=json.load(sys.stdin); assert value["ok"] is True and value["data"]["removed"] is True' <<<"$remove_json"
trust_target=""
if find /usr/local/share/ca-certificates -maxdepth 1 -name "umec-nav-studio-${fingerprint:0:16}.crt" -print -quit | grep -q .; then
  echo "Connector-owned certificate remained after removal" >&2
  exit 1
fi
if curl --fail --silent --show-error --max-time 3 "https://localhost:$port/" >/dev/null 2>&1; then
  echo "Disposable certificate remained trusted after removal" >&2
  exit 1
fi

printf '{"ok":true,"package":"%s","binary":"%s","fingerprint":"%s","gui_smoke_seconds":5,"trust_cleanup":true}\n' \
  "$package_name" "$binary" "$fingerprint"
