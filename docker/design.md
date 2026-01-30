docker/
  compose/
    compose.base.yml            # common network, volumes, shared defaults
    compose.dev.yml             # dev-friendly (ports exposed, hot reload optional)
    compose.ci.yml              # deterministic runs for CI later
  services/
    redis/
      Dockerfile                # optional; otherwise use official image
      redis.conf                # optional
    valkey/
      Dockerfile                # optional; otherwise use official image
  scripts/
    up                          # wrapper script: profiles, args
    down
    logs
    reset                       # remove volumes
    exec-redis
  env/
    dev.env                     # REDIS_URL, etc. (not secrets)