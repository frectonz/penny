default:
  @just --choose

run:
  cargo r -- serve penny.toml

check-app1:
  curl http://localhost:3030/api/status -H "Host: app1.local"

check-app2:
  curl http://localhost:3030/api/status -H "Host: app2.local"
