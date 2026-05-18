# Safety

`kalx` is demo-first.

Write command rules:

- no `--live`: preview only, no API mutation
- `--live` on `demo`: send the mutation
- `--live --yes` on `prod`: required

This applies to:

- `orders create`
- `orders cancel`
- `orders cancel-market`
- `orders amend`

Preview mode prints a normalized JSON summary of the action that would be taken.

