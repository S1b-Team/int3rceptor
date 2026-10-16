# Capture storage encryption

Int3rceptor stores captured traffic in SQLite (`INTERCEPTOR_DB_PATH`, default
`data/interceptor.sqlite`). Request and response **headers and bodies** are
encrypted per field with AES-256-GCM using a 256-bit master key and a random
96-bit nonce per field.

These columns stay in plaintext so filtering and search keep working:
id, timestamp, method, **full URL (including the query string)**, TLS flag,
status code and duration. Don't rely on encryption for secrets that travel in
URLs.

## Setting up a key

The proxy refuses to start without a valid key, unless plaintext storage is
explicitly enabled (see below).

Recommended: keep the key in a file only you can read, and point
`INTERCEPTOR_ENCRYPTION_KEY_FILE` at it.

```bash
mkdir -p ~/.config/int3rceptor
( umask 077 && openssl rand -hex 32 > ~/.config/int3rceptor/key )
export INTERCEPTOR_ENCRYPTION_KEY_FILE=~/.config/int3rceptor/key
```

- The file must contain exactly 64 hex characters. Surrounding whitespace and
  a trailing newline are ignored.
- **Back the key up.** Without it, stored captures can't be read.
- Never commit it. `.interceptor_key` is in `.gitignore`, but a path outside
  the repository is safer.
- `INTERCEPTOR_ENCRYPTION_KEY` (the hex key itself) also works, but
  environment variables leak more easily (process listings, `/proc/<pid>/environ`,
  shell history, child processes). Prefer the file.

### Resolution order

1. `INTERCEPTOR_ENCRYPTION_KEY`: used if set. An invalid value is a startup error.
2. `INTERCEPTOR_ENCRYPTION_KEY_FILE`: used if set. A missing or invalid file is a startup error.
3. `./.interceptor_key` in the working directory: used if it exists. An invalid file is a startup error.
4. `INTERCEPTOR_UNENCRYPTED=1` (or `true`): plaintext storage.
5. Otherwise startup fails with "No encryption key found".

An invalid source never falls through to the next one.

## Plaintext mode (testing only)

`INTERCEPTOR_UNENCRYPTED` accepts only `1` or `true` (case-insensitive). Any
other value, including `0` and `false`, is rejected at startup so it can't
silently turn encryption off. When plaintext mode is active, a warning is
logged at startup. Every header, cookie, auth token and body is then written
to the database file unencrypted.

## Reading with the wrong key

If a stored field can't be decrypted (wrong key, or a database written in
plaintext mode and opened with a key), the entry is still returned, but the
affected fields are empty. A request header
`x-interceptor-decrypt-error: <fields>` is added, and a warning is logged with
the capture id.

## File permissions (Unix)

- A newly created database directory gets mode `0700`, and a new database file gets `0600`.
  Existing paths are not changed.
- The CA directory (`~/.interceptor/ca`) gets mode `0700` when it is created, and the CA private key
  (`ca_key.pem`) is written with mode `0600`. An existing key that other users can read is
  tightened to `0600` on startup, with a warning.

## Known limitations / follow-ups

- No associated data binds ciphertext to its row and column, so encrypted
  values could be swapped between rows undetected.
- No key rotation, and no migration between plaintext and encrypted databases.
- URLs (including query strings) are stored unencrypted.
