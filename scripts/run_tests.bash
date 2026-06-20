#!/usr/bin/env bash

mkdir -p /tmp/regression
rm -rf /tmp/regression/*
cp -r examples /tmp/regression/examples

echo "=== Psy Tests ==="
echo

passed=0
failed=0
failed_names=()

for f in $(find /tmp/regression/examples -name "*.psy"); do
    dir=$(dirname "$f")
    base=$(basename "$f")
    ignore_file="$dir/.regression-ignore"

    name=$(realpath --relative-to=/tmp/regression/examples "$f")

    if [[ "$name" == prototypes/* ]]; then
        continue
    fi

    if [ -f "$ignore_file" ] && grep -qxF "$base" "$ignore_file"; then
        continue
    fi

    stderr_output=$(timeout 5 ./target/release/psy "$f" 2>&1 1>/dev/null)
    issue=$(echo "$stderr_output" | grep -i -E "error|panic|runtime")

    if [[ "$name" == errors/* ]]; then
        if [ -n "$issue" ]; then
            echo "✓ $name (expected failure)"
            passed=$((passed + 1))
        else
            echo "✗ $name (expected to fail, but didn't)"
            failed=$((failed + 1))
            failed_names+=("$name")
        fi
        continue
    fi

    if [ -z "$issue" ]; then
        echo "✓ $name"
        passed=$((passed + 1))
    else
        echo "✗ $name"
        echo "$issue" | sed 's/^/    /'
        failed=$((failed + 1))
        failed_names+=("$name")
    fi
done

echo
echo "=== Results: $passed passed, $failed failed ==="
if [ ${#failed_names[@]} -gt 0 ]; then
    echo "Failed: ${failed_names[*]}"
    exit 1
fi

exit 0