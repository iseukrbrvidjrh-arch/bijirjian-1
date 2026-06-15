#!/bin/zsh

# Second Brain OS 本地启动脚本
# 双击这个文件，就会自动进入项目目录并启动应用

PROJECT_DIR="$HOME/Documents/Codex/2026-06-13/role-15-ai-local-first-project"

echo "======================================"
echo "正在启动 Second Brain OS..."
echo "项目目录：$PROJECT_DIR"
echo "======================================"
echo ""

# 检查项目目录是否存在
if [ ! -d "$PROJECT_DIR" ]; then
  echo "错误：找不到项目目录。"
  echo "请检查路径是否正确："
  echo "$PROJECT_DIR"
  echo ""
  echo "按任意键退出..."
  read -k 1
  exit 1
fi

cd "$PROJECT_DIR" || exit 1

# 检查 pnpm 是否已安装
if ! command -v pnpm >/dev/null 2>&1; then
  echo "错误：未检测到 pnpm。"
  echo "请先安装 pnpm 后再启动。"
  echo ""
  echo "可以尝试执行："
  echo "npm install -g pnpm"
  echo ""
  echo "按任意键退出..."
  read -k 1
  exit 1
fi

# 检查 package.json 是否存在
if [ ! -f "package.json" ]; then
  echo "错误：当前目录不是正确的项目根目录。"
  echo "没有找到 package.json。"
  echo ""
  echo "按任意键退出..."
  read -k 1
  exit 1
fi

echo "环境检查通过。"
echo "正在启动桌面应用..."
echo ""

pnpm tauri dev

echo ""
echo "Second Brain OS 已退出。"
echo "按任意键关闭窗口..."
read -k 1
