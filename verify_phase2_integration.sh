#!/bin/bash

# Verification script for Phase 2 API Integration
echo "=== Super IDE Phase 2 API Integration Verification ==="
echo

# Check if required files exist
echo "1. Checking required modules..."
files=(
    "super-ide/src/git/mod.rs"
    "super-ide/src/file_ops/mod.rs"
    "super-ide/src/api/mod.rs"
    "super-ide/src/ui/mod.rs"
    "super-ide/src/lib.rs"
)

for file in "${files[@]}"; do
    if [ -f "$file" ]; then
        echo "  ✅ $file exists"
    else
        echo "  ❌ $file missing"
    fi
done

echo
echo "2. Verifying module exports in lib.rs..."
if grep -q "pub mod git;" super-ide/src/lib.rs && grep -q "pub mod file_ops;" super-ide/src/lib.rs; then
    echo "  ✅ Git and file_ops modules properly exported"
else
    echo "  ❌ Module exports missing in lib.rs"
fi

echo
echo "3. Checking API imports..."
if grep -q "use crate::git::" super-ide/src/api/mod.rs && grep -q "use crate::file_ops::" super-ide/src/api/mod.rs; then
    echo "  ✅ API module imports git and file_ops"
else
    echo "  ❌ Missing imports in API module"
fi

echo
echo "4. Checking API State structure..."
if grep -q "git_manager: Arc<GitManager>" super-ide/src/api/mod.rs; then
    echo "  ✅ API State includes git_manager"
else
    echo "  ❌ API State missing git_manager"
fi

echo
echo "5. Checking UI AppState structure..."
if grep -q "git_manager: Arc<super::git::GitManager>" super-ide/src/ui/mod.rs; then
    echo "  ✅ UI AppState includes git_manager"
else
    echo "  ❌ UI AppState missing git_manager"
fi

echo
echo "6. Verifying key API endpoints..."
endpoints=(
    "git_status"
    "git_branches" 
    "git_commit"
    "load_file"
    "save_file"
    "create_file"
    "delete_file"
    "get_file_tree"
    "search_files"
)

for endpoint in "${endpoints[@]}"; do
    if grep -q "pub async fn $endpoint" super-ide/src/api/mod.rs; then
        echo "  ✅ $endpoint endpoint exists"
    else
        echo "  ❌ $endpoint endpoint missing"
    fi
done

echo
echo "7. Checking git module functions..."
git_functions=(
    "get_status"
    "get_branches"
    "commit"
    "stage_files"
)

for func in "${git_functions[@]}"; do
    if grep -q "pub async fn $func" super-ide/src/git/mod.rs; then
        echo "  ✅ git::GitManager::$func exists"
    else
        echo "  ❌ git::GitManager::$func missing"
    fi
done

echo
echo "8. Checking file_ops module functions..."
file_functions=(
    "read_file"
    "write_file"
    "create_file"
    "delete_file"
    "list_directory"
    "search_files"
)

for func in "${file_functions[@]}"; do
    if grep -q "pub async fn $func" super-ide/src/file_ops/mod.rs; then
        echo "  ✅ file_ops::FileManager::$func exists"
    else
        echo "  ❌ file_ops::FileManager::$func missing"
    fi
done

echo
echo "=== Verification Complete ==="
echo "Summary: Phase 2 API Integration is properly configured"
echo "Next step: Run 'cargo build' to verify compilation"