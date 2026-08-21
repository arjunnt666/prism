# Status

## diagnosis
Only repo that had hard-failing CI (no continue-on-error). Failure was a bad unicode escape in sample titles (`\u2014` vs `\u{2014}`).

## fixed
- sample title format string compiles
- unused import cleaned

## works today
- offline sample provider
- rank diff, features, competitor maps
- memory + json dir stores
- `prism capture|diff|features|competitors`
- unit tests in collect/parse/features/store/diff/analyze

## still not production
- live public SERP HTML will break parsers when markup changes
- no durable research DB beyond json files
