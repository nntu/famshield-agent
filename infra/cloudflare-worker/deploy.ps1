<#
.SYNOPSIS
  Script tự động cài đặt và triển khai Cloudflare Worker cho FamShield Version Checker
  Single Subdomain: famshield.ttcantho.io.vn (Không sinh thêm subdomain)

.DESCRIPTION
  Yêu cầu: Node.js (npm / npx) và tài khoản Cloudflare đã đăng nhập (npx wrangler login)
#>

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  FAMSHIELD CLOUDFLARE WORKER (famshield.ttcantho.io.vn)  " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

$WorkerDir = $PSScriptRoot
Set-Location $WorkerDir

# 1. Cài đặt dependencies nếu chưa có
if (-not (Test-Path "node_modules")) {
    Write-Host "[1/3] Cài đặt dependencies (wrangler & types)..." -ForegroundColor Yellow
    npm install
}

# 2. Kiểm tra cấu hình Cloudflare Wrangler
Write-Host "[2/3] Kiểm tra cấu hình Cloudflare Wrangler..." -ForegroundColor Yellow
Write-Host "  File cấu hình: wrangler.jsonc" -ForegroundColor Gray
Write-Host "  KV Binding   : FAMSHIELD_VERSION_STORE" -ForegroundColor Gray
Write-Host "  Domain route : famshield.ttcantho.io.vn/version.json" -ForegroundColor Gray
Write-Host "  Domain route : famshield.ttcantho.io.vn/v1/release/*" -ForegroundColor Gray
Write-Host "  Domain route : famshield.ttcantho.io.vn/download/*" -ForegroundColor Gray

# 3. Triển khai lên Cloudflare Edge
Write-Host "[3/3] Tiến hành triển khai worker..." -ForegroundColor Green
npx wrangler deploy

Write-Host "`n[HOÀN TẤT] Worker đã sẵn sàng hoạt động tại:" -ForegroundColor Green
Write-Host "  - Kiểm tra phiên bản : https://famshield.ttcantho.io.vn/version.json" -ForegroundColor White
Write-Host "  - Link tải nhanh     : https://famshield.ttcantho.io.vn/download/latest" -ForegroundColor White
Write-Host "  - Kiểm tra sức khỏe  : https://famshield.ttcantho.io.vn/health" -ForegroundColor White
