# kea leases viewer

**WIP.**  
A simple HTTP server that serves up HTML pages containing the current leases of
the `kea` DHCP server.


## Configuration

The following environment variables can be defined to overwrite default
behaviour:
```bash
BIND_ADDR="0.0.0.0:80" # The socket address that the HTTP server binds to
LEASES_FILE="/var/db/kea/dhcp4.leases" # The dhcp4.leases file location
```
