# Agent
## Configuration
The agent app can be configured by a TOML file, this must exist in the directory where the agent is launched. It also must be called `agent.toml`. Example shown below:

```toml
# what ip to bind to, use 0.0.0.0 for all
host="127.0.0.1"
# port to listen on
port=8080
# enable if using a reverse proxy so real client ip is forwarded
using_proxy = false

[authentication]
# Whether to only allow registed ip's
check_ip = true
allowed_ip = ["127.0.0.1"]

# Whether to only allow clients that have a valid authentication key
check_key = true
allowed_keys = ["testing123"]
```

## API
Each agent serves a HTTP API allowing for requesting statistics.

### Authentication
If agent is configured to require key authentication, the client must send a Authorization header. For example:

```
Authorization: Bearer testing123
```

### Routes

#### /
#### /cpu
#### /memory
