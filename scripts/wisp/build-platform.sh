#!/usr/bin/env bash
# WispLauncher 分平台构建脚本
#
# target 目录结构：
#   target/release/<Platform>/        最终安装包（deb/rpm/nsis .exe）与裸二进制，不含中间文件
#   target/debug/<Platform>/          debug 裸二进制
#   target/intermediate/<Platform>/   cargo 编译中间目录（CARGO_TARGET_DIR）
#
# 用法：
#   ./scripts/wisp/build-platform.sh linux  release   # 构建 Linux 发布版（deb+rpm）
#   ./scripts/wisp/build-platform.sh linux  debug
#   ./scripts/wisp/build-platform.sh windows release   # 构建 Windows 发布版（nsis）
#   ./scripts/wisp/build-platform.sh windows debug
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
APP_DIR="$ROOT/apps/app"

PLATFORM="${1:-}"
PROFILE="${2:-release}"

if [[ "$PLATFORM" != "linux" && "$PLATFORM" != "windows" ]]; then
	echo "用法: $0 {linux|windows} {release|debug}" >&2
	exit 1
fi

# 平台信息
declare -A TRIPLE=(
	[linux]="aarch64-unknown-linux-gnu"
	[windows]="x86_64-pc-windows-gnu"
)
declare -A HUMAN=(
	[linux]="Linux"
	[windows]="Windows"
)

# 发布平台的打包类型（仅 release 生效）
declare -A BUNDLE_TARGETS=(
	[linux]="deb,rpm"
	[windows]="nsis"
)

# 目标中间路径：$CARGO_TARGET_DIR 由 cargo 决定 profile 子目录，
# 交叉编译时还会追加 triple 子目录，因此这里不预设 profile 段。
BUILD_TARGET_DIR="$ROOT/target/intermediate/${HUMAN[$PLATFORM]}"
OUT_DIR="$ROOT/target/$PROFILE/${HUMAN[$PLATFORM]}"
mkdir -p "$OUT_DIR"

echo "==> 平台: ${HUMAN[$PLATFORM]} / $PROFILE"
echo "==> 中间目录: $BUILD_TARGET_DIR"
echo "==> 输出目录: $OUT_DIR"

export CARGO_TARGET_DIR="$BUILD_TARGET_DIR"

# ---- 统一构建入口 ----
# 各平台附加的 tauri config 文件（nsis 语言、deb/rpm targets 等在平台配置中）
CONFIG_ARG=()
if [[ "$PLATFORM" == "linux" ]]; then
	CONFIG_ARG=(--config tauri.linux.conf.json)
elif [[ "$PLATFORM" == "windows" ]]; then
	CONFIG_ARG=(--config tauri.windows.conf.json)
fi

# Linux native：不加 --target；Windows 交叉：加 --target <triple>
CARGO_TAURI_ARGS=()
if [[ "$PLATFORM" == "windows" ]]; then
	CARGO_TAURI_ARGS+=(--target "${TRIPLE[$PLATFORM]}")
fi

if [[ "$PROFILE" == "release" ]]; then
	CARGO_TAURI_ARGS+=(--bundles "${BUNDLE_TARGETS[$PLATFORM]}")
else
	CARGO_TAURI_ARGS+=(--debug --no-bundle)
fi

echo "==> 执行: node tauri build ${CONFIG_ARG[*]:-} ${CARGO_TAURI_ARGS[*]}"
(
	cd "$APP_DIR"
	node ~/.local/share/pi-node/node-v22.23.2-linux-arm64/bin/tauri build "${CONFIG_ARG[@]}" "${CARGO_TAURI_ARGS[@]}"
)

# ---- 采集产物到 OUT_DIR ----
# 交叉编译时 cargo 会在 $CARGO_TARGET_DIR/<triple>/<profile> 生成产物
ARTIFACT_ROOT="$BUILD_TARGET_DIR"
if [[ "$PLATFORM" == "windows" ]]; then
	ARTIFACT_ROOT="$ARTIFACT_ROOT/${TRIPLE[$PLATFORM]}"
fi
PROFILE_DIR="$ARTIFACT_ROOT/$PROFILE"

echo "==> 采集中间产物: $PROFILE_DIR"

# 1. 裸二进制
if [[ "$PLATFORM" == "linux" ]]; then
	BIN="$PROFILE_DIR/WispLauncher"
	[[ -f "$BIN" ]] || BIN="$PROFILE_DIR/theseus_gui"
	if [[ -f "$BIN" ]]; then
		if [[ "$PROFILE" == "release" ]]; then
			cp -f "$BIN" "$OUT_DIR/WispLauncher"
		else
			cp -f "$BIN" "$ROOT/target/debug/Linux/WispLauncher"
		fi
		echo "    - 裸二进制 -> $OUT_DIR/WispLauncher"
	fi
elif [[ "$PLATFORM" == "windows" ]]; then
	BIN="$PROFILE_DIR/WispLauncher.exe"
	if [[ -f "$BIN" ]]; then
		if [[ "$PROFILE" == "release" ]]; then
			cp -f "$BIN" "$OUT_DIR/WispLauncher.exe"
		else
			cp -f "$BIN" "$ROOT/target/debug/Windows/WispLauncher.exe"
		fi
		echo "    - 裸程序 -> $OUT_DIR/WispLauncher.exe"
	fi
fi

# 2. 安装包（仅 release）
if [[ "$PROFILE" == "release" ]]; then
	if [[ "$PLATFORM" == "linux" ]]; then
		cp -f "$PROFILE_DIR/bundle/deb/"*.deb "$OUT_DIR/" 2>/dev/null || true
		cp -f "$PROFILE_DIR/bundle/rpm/"*.rpm "$OUT_DIR/" 2>/dev/null || true
		echo "    - deb/rpm 安装包"
	elif [[ "$PLATFORM" == "windows" ]]; then
		cp -f "$PROFILE_DIR/bundle/nsis/"*.exe "$OUT_DIR/" 2>/dev/null || true
		echo "    - NSIS 安装包"
	fi
fi

echo "==> 完成。产物在: $OUT_DIR"
ls -la "$OUT_DIR"