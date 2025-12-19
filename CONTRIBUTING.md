# Contributing

    make test          # host tests, gcc or clang (HOSTCC=clang), add SAN=1 for asan+ubsan
    make lint          # cppcheck + clippy
    make smoke         # build and boot under qemu

CI runs the same things (`.github/workflows/ci.yml`). The C code that has host
tests (`ipc`, `pmap`, `vm_page`) is where regressions are cheapest to catch, so
new logic there should come with a test in `tests/`.

Commit messages: `area: what changed`, imperative, with the reason in the body
when it is not obvious (for example *why* a fix is needed, not just what it does).
