# Redis RESP Parser Port

This crate is a validation-only Rust port of a narrow Redis request parser slice.
It parses complete RESP2 multibulk command frames into owned command argument
bytes and deliberately excludes networking, command execution, replies, and Redis
server state.