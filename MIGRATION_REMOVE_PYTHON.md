# Migration note: removing legacy Python

DiscOS is primarily Rust.

If your existing DiscOS GitHub repo contains a legacy Python implementation,
remove it before pushing this Rust workspace.

Suggested approach:

```bash
# Example (adjust paths)
git rm -r --cached python/ src_py/ notebooks/ || true
find . -name "*.py" -o -name "requirements.txt" -o -name "Pipfile" -o -name "poetry.lock" \
  | xargs -I{} git rm --cached "{}" || true
```

Then add this workspace and commit.

Note: this repo retains `examples/python_ipc/` as an **interoperability demo**.
If you want a 100% Rust-only DiscOS, you can remove that folder.
