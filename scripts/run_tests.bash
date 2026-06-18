#!/usr/bin/env bash

mkdir -p /tmp/psc_regression
rm -rf /tmp/psc_regression/*
cp -r examples /tmp/psc_regression/examples

echo "=== Pseudocode Tests ==="
echo

passed=0
failed=0
failed_names=()

for f in $(find /tmp/psc_regression/examples -name "*.psc"); do
    dir=$(dirname "$f")
    base=$(basename "$f")
    ignore_file="$dir/.regression-ignore"

    name=$(realpath --relative-to=/tmp/psc_regression/examples "$f")

    # prototypes/ contains examples for modules that don't exist yet
    # skip entirely rather than scoring as failures.
    if [[ "$name" == prototypes/* ]]; then
        continue
    fi

    if [ -f "$ignore_file" ] && grep -qxF "$base" "$ignore_file"; then
        continue
    fi

    output=$(timeout 5 ./target/release/psc "$f" 2>&1)
    issue=$(echo "$output" | grep -i -E "error|panic")

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
fi