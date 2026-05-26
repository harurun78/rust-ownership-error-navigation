# Redis Upstream

## Selected Source

- Repository: `https://github.com/redis/redis.git`
- Tag: `7.2.4`
- Commit: `d2c8a4b91e8c0e6aefd1f5bc0bf582cddbe046b7`
- License: BSD-3-Clause, from upstream `COPYING`
- Local checkout: `validation/ports/redis/upstream/redis/`

## Acquisition Commands

```sh
mkdir -p validation/ports/redis/upstream
git clone --depth 1 --branch 7.2.4 https://github.com/redis/redis.git validation/ports/redis/upstream/redis
git -C validation/ports/redis/upstream/redis rev-parse HEAD
git -C validation/ports/redis/upstream/redis describe --tags --exact-match
```

## Verification

```sh
git check-ignore -v validation/ports/redis/upstream/redis/src/networking.c
```

The upstream checkout is intentionally ignored by Git. Only this metadata file should be tracked.

## Initial Porting Focus

The first validation target is request parsing from `src/networking.c`:

- `processInputBuffer`
- `processInlineBuffer`
- `processMultibulkBuffer`

The implementation will not port Redis networking, command execution, server state, persistence, replication, cluster, modules, or ACL behavior.
