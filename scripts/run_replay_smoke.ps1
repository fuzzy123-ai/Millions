$ErrorActionPreference = "Stop"

& C:\Users\nkatz\.cargo\bin\cargo.exe test replay::

Write-Host "replay_smoke status=ok scope=server_replay"
