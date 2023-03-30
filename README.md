# apache-mod-status-parser

A parser for Apache mod_status page. Prints out JSON.

## Example

```
curl localhost/server-status \
    | apache-mod-status-parser \
    | jq -r '.workers[] \
    | "\(.vhost)\t\(.request)"'
```

## TODO

- [ ] Some fields are missing.
- [ ] `HAS_TIMES` is assumed.
- [ ] Better error handling.
- [ ] Tests.