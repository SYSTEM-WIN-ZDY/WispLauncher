#!/bin/bash
# WispLauncher startup script
# 禁用 WebKitGTK 的 DMA-BUF renderer，避免在部分 ARM/嵌入式 GPU 环境下
# 启动后发生堆损坏崩溃 (free(): corrupted unsorted chunks)。
export WEBKIT_DISABLE_DMABUF_RENDERER=1
cd /home/tiny/桌面/Wisp/WispLauncher
exec ./target/debug/theseus_gui "$@"
