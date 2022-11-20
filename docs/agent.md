# Agent
The agent should be run on each device you want to monitor. If metrics are to be sent over the network, a port must be opened to allow a gathering app to request metrics. Optional webhooks can also be sent by the agent for critical events. To minimise server load, the agent caches most recent metrics data.

## Configuration
The agent app can be configured by a TOML file, this must exist in the directory where the agent is launched. It also must be called `agent.toml`. Example shown below:

```toml
# The unique name for the agent, if not given a uuid4 will be generated
id = "agent-abc123"
# what ip to bind to, use 0.0.0.0 for all
host="127.0.0.1"
# port to listen on
port=8080
# enable if using a reverse proxy so real client ip is forwarded
using_proxy = false
# duration to cache metrics for future requests in seconds
cache_for = 2


[certificates]
# Path to certificates, enables serving on HTTPS
# Expects files in PEM format
private_path = "key.pem"
public_path = "cert.pem"


[authentication]
# Whether to only allow registed ip's
check_ip = true
allowed_ip = ["127.0.0.1"]

# Whether to only allow clients that have a valid authentication key
check_key = true
allowed_keys = ["testing123"]


# Send event via webhooks to clients
[webhooks]
# When server starts
[[webhooks.on_start]]
# Where to send hook
url = "http://localhost:8888/my-hook"
# Optional secret to sign the request body using X-Hub-Signature-256
secret = "my_secret"
```

## API
Each agent serves a HTTP API allowing for requesting statistics.

### Features
- Optional whitelisted IP list
- Optional token authentication
- Multiple routes to get specific data to minimise response size
- Can be served over HTTPS
- Response body sent via JSON

### Authentication
If agent is configured to require key authentication, the client must send a Authorization header. For example:

```
Authorization: Bearer testing123
```

### Routes
Documented in the OpenAPI spec in [openapi.yml](openapi.yml)

## Webhooks
If was built with webhooks support, agent will support sending webhooks to external devices.

### Features
- Body is sent as JSON
- Timestamped
- Optional body signing to reduce replay attacks (using X-Hub-Signature-256)
- Sent over HTTP/S
- Support can be completely removed during agent build process

### Hooks
#### on_start
When the agent starts.
