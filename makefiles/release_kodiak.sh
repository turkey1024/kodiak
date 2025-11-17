#!/bin/sh
#
# release_kodiak.sh
#
# SPDX-FileCopyrightText: 2024 Softbear, Inc.
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# This script pushes the contents of the working directory to GitHub.

set -e  # Exit on any error

REPO_NAME="kodiak"

# 验证仓库函数
validate_repository() {
    if [ ! -e .git ]; then
        echo "./.git not found: must run in root of $REPO_NAME repository"
        exit 1
    fi

    # 检查远程仓库是否包含 kodiak（不区分大小写）
    if git remote -v 2>/dev/null | grep -i "kodiak" > /dev/null; then
        echo "Repo name: $REPO_NAME (verified via git remote)"
        return 0
    fi

    # 或者检查当前目录名是否为 kodiak
    if basename "$(pwd)" | grep -q -i "^kodiak$"; then
        echo "Repo name: $REPO_NAME (verified via directory name)"
        return 0
    fi

    # 或者检查是否存在项目特定的文件（如 Cargo.toml）
    if [ -f "Cargo.toml" ]; then
        echo "Repo name: $REPO_NAME (verified via Cargo.toml presence)"
        return 0
    fi

    echo "must run in $REPO_NAME repository"
    exit 1
}

# 使用说明
USAGE="usage: $0 kodiak_tag action"
KODIAK_TAG="$1"
if [ -z "$KODIAK_TAG" ]; then
    echo "$USAGE"
    exit 1
fi

ACTION="$2"

# 强制使用家目录下的临时目录（Termux 修复）
RELEASE_BASE="$HOME/.kodiak_releases"
mkdir -p "$RELEASE_BASE"
TMPDIR="$RELEASE_BASE/release_${REPO_NAME}_$$"

# 清理函数
cleanup() {
    if [ -d "$TMPDIR" ] && [ "$ACTION" = "push" ]; then
        echo "Cleaning up temporary directory: $TMPDIR"
        rm -rf "$TMPDIR"
    elif [ -d "$TMPDIR" ] && [ "$ACTION" = "preview" ]; then
        echo "Preview files kept in: $TMPDIR"
    fi
}

# 设置退出时清理
trap cleanup EXIT

# 验证仓库
validate_repository

if [ -d "$TMPDIR" ]; then
    echo "Clearing existing temporary directory: $TMPDIR"
    rm -rf "$TMPDIR"
fi

echo "Kodiak tag: $KODIAK_TAG"
echo "Action: $ACTION"
echo "Temporary directory: $TMPDIR"

# 使用简单的文件复制而不是 git clone（避免权限问题）
echo "Creating release directory structure"
mkdir -p "$TMPDIR"

# 复制 .git 目录（手动创建git仓库）
echo "Initializing git repository"
cp -r .git "$TMPDIR/" 2>/dev/null || {
    # 如果复制.git失败，初始化新的git仓库
    cd "$TMPDIR"
    git init
    git remote add origin "$(git -C . remote get-url origin 2>/dev/null || echo "git@github.com:turkey1024/kodiak.git")"
    cd - >/dev/null
}

# 复制文件，排除不需要的目录和文件
echo "Copying files to repo"
EXCLUDE_LIST=(
    --exclude=".git"
    --exclude=".github"
    --exclude=".cargo"
    --exclude=".ssh"
    --exclude=".vscode"
    --exclude="target"
    --exclude="Cargo.lock"
    --exclude=".gitlab-ci.yml"
    --exclude="archive"
    --exclude="manifest"
    --exclude="sprite_sheet_util"
    --exclude="uploader"
    --exclude="*.bak"
    --exclude="makefiles/release_kodiak.sh"
)

rsync -rlptv "${EXCLUDE_LIST[@]}" ./ "$TMPDIR/"

# 清理备份文件
echo "Removing .bak files if any"
find "$TMPDIR" -name '*.bak' -delete 2>/dev/null || true

# 更新 Cargo.toml 版本
echo "Editing Cargo.toml files to have version $KODIAK_TAG"
find "$TMPDIR" -name 'Cargo.toml' -exec sed -i.bak -e "1,7s/^version[[:space:]]*=[[:space:]]*\"[^\"]*\"/version = \"${KODIAK_TAG}\"/" {} \;

# 清理 sed 创建的备份文件
find "$TMPDIR" -name '*.bak' -delete

# 验证版本更新
echo "Verifying version updates:"
find "$TMPDIR" -name 'Cargo.toml' -exec grep -H '^version[[:space:]]*=' {} \;

# 提交更改
echo "Committing changes"
cd "$TMPDIR"

# 设置用户信息
if [ -z "$(git config user.name)" ]; then
    git config user.name "Release Script"
    git config user.email "release@example.com"
fi

# 添加所有文件
git add -A

if git diff --cached --quiet; then
    echo "No changes to commit"
else
    git commit -m "Release $KODIAK_TAG"
    echo "Changes committed"
fi

# 创建标签
if git tag -f "$KODIAK_TAG" 2>/dev/null; then
    echo "Tag $KODIAK_TAG created/updated"
else
    git tag "$KODIAK_TAG"
    echo "Tag $KODIAK_TAG created"
fi

echo "Ready to push"

# 根据操作类型执行
case "$ACTION" in
    "push")
        echo "Pushing to GitHub"
        # 尝试推送到 main 或 master 分支
        if git push -f origin main 2>/dev/null || git push -f origin master; then
            git push --tags
            echo "Push completed successfully"
        else
            echo "Error: Failed to push to repository"
            exit 1
        fi
        ;;
    "preview"|"")
        echo "Preview mode: edited version is in $TMPDIR"
        echo "To push manually, run:"
        echo "  cd $TMPDIR && git push && git push --tags"
        ;;
    *)
        echo "Unknown action: $ACTION"
        echo "Valid actions: push, preview"
        echo "Preview mode: edited version is in $TMPDIR"
        ;;
esac

