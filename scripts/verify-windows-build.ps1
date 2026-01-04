# Windows Build Verification Script
# Verifies Windows-specific build requirements and potential issues

param(
    [string]$BuildPath = "target\release\bundle",
    [switch]$Verbose
)

Write-Host "🪟 Windows Build Verification" -ForegroundColor Cyan
Write-Host "============================" -ForegroundColor Cyan
Write-Host ""

# Function to print colored output
function Write-Status {
    param(
        [string]$Status,
        [string]$Message
    )
    
    switch ($Status) {
        "success" { Write-Host "✅ $Message" -ForegroundColor Green }
        "error" { Write-Host "❌ $Message" -ForegroundColor Red }
        "warning" { Write-Host "⚠️  $Message" -ForegroundColor Yellow }
        "info" { Write-Host "ℹ️  $Message" -ForegroundColor Blue }
    }
}

# Check Windows version compatibility
Write-Host "🖥️  Checking Windows compatibility:" -ForegroundColor White
$winVersion = [System.Environment]::OSVersion.Version
$minVersion = [Version]"10.0.18362"  # Windows 10 1903

if ($winVersion -ge $minVersion) {
    Write-Status "success" "Windows version compatible: $($winVersion.ToString())"
} else {
    Write-Status "warning" "Windows version may have compatibility issues: $($winVersion.ToString())"
    Write-Status "info" "Minimum recommended: Windows 10 version 1903 (10.0.18362)"
}
Write-Host ""

# Check WebView2 Runtime
Write-Host "🌐 Checking WebView2 Runtime:" -ForegroundColor White
try {
    $webview2 = Get-AppxPackage -Name "Microsoft.WebView2" -ErrorAction SilentlyContinue
    if ($webview2) {
        Write-Status "success" "WebView2 Runtime installed: $($webview2.Version)"
    } else {
        # Check registry for WebView2
        $regPath = "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
        if (Test-Path $regPath) {
            $version = (Get-ItemProperty $regPath -Name "pv" -ErrorAction SilentlyContinue).pv
            if ($version) {
                Write-Status "success" "WebView2 Runtime installed: $version"
            } else {
                Write-Status "warning" "WebView2 Runtime status unclear"
            }
        } else {
            Write-Status "error" "WebView2 Runtime not found"
            Write-Status "info" "Download from: https://developer.microsoft.com/microsoft-edge/webview2/"
        }
    }
} catch {
    Write-Status "warning" "Could not verify WebView2 installation: $($_.Exception.Message)"
}
Write-Host ""

# Check Visual C++ Redistributables
Write-Host "🔧 Checking Visual C++ Redistributables:" -ForegroundColor White
$vcRedist = @(
    "Microsoft Visual C++ 2015-2022 Redistributable (x64)",
    "Microsoft Visual C++ 2015-2022 Redistributable (x86)",
    "Microsoft Visual C++ 2019 Redistributable (x64)",
    "Microsoft Visual C++ 2019 Redistributable (x86)"
)

$foundRedist = $false
foreach ($redist in $vcRedist) {
    $installed = Get-WmiObject -Class Win32_Product | Where-Object { $_.Name -like "*$redist*" }
    if ($installed) {
        Write-Status "success" "$redist installed"
        $foundRedist = $true
        break
    }
}

if (-not $foundRedist) {
    Write-Status "warning" "Visual C++ Redistributables not found"
    Write-Status "info" "Usually pre-installed on modern Windows systems"
}
Write-Host ""

# Check build outputs
Write-Host "📦 Checking build outputs:" -ForegroundColor White
if (Test-Path $BuildPath) {
    Write-Status "success" "Build directory found: $BuildPath"
    
    # Check for NSIS installer
    $nsisPath = Join-Path $BuildPath "nsis"
    if (Test-Path $nsisPath) {
        $nsisFiles = Get-ChildItem $nsisPath -Filter "*.exe" -ErrorAction SilentlyContinue
        if ($nsisFiles) {
            Write-Status "success" "NSIS installer found: $($nsisFiles[0].Name)"
            
            # Check file size
            $sizeKB = [math]::Round($nsisFiles[0].Length / 1KB, 1)
            Write-Host "    📏 Size: $sizeKB KB" -ForegroundColor Gray
            
            # Check digital signature (SSL.com specific)
            try {
                $signature = Get-AuthenticodeSignature $nsisFiles[0].FullName
                if ($signature.Status -eq "Valid") {
                    Write-Status "success" "Digital signature valid"
                    Write-Host "    🔐 Signer: $($signature.SignerCertificate.Subject)" -ForegroundColor Gray

                    # Check if it's SSL.com certificate
                    if ($signature.SignerCertificate.Subject -like "*SSL.com*" -or $signature.SignerCertificate.Issuer -like "*SSL.com*") {
                        Write-Status "success" "SSL.com certificate detected"
                    }

                    # Check timestamp
                    if ($signature.TimeStamperCertificate) {
                        Write-Host "    ⏰ Timestamped: $($signature.TimeStamperCertificate.Subject)" -ForegroundColor Gray
                        if ($signature.TimeStamperCertificate.Subject -like "*SSL.com*") {
                            Write-Status "success" "SSL.com timestamp server used"
                        }
                    } else {
                        Write-Status "warning" "Signature not timestamped"
                    }

                    # SmartScreen compatibility
                    Write-Status "success" "Should pass Windows Defender SmartScreen"
                } elseif ($signature.Status -eq "NotSigned") {
                    Write-Status "warning" "Executable is not digitally signed"
                    Write-Status "info" "Users will see SmartScreen warnings"
                } else {
                    Write-Status "error" "Digital signature invalid: $($signature.Status)"
                    Write-Host "    ❌ Status message: $($signature.StatusMessage)" -ForegroundColor Red
                }
            } catch {
                Write-Status "warning" "Could not verify digital signature: $($_.Exception.Message)"
            }
        } else {
            Write-Status "error" "No NSIS installer found in $nsisPath"
        }
    } else {
        Write-Status "warning" "NSIS build directory not found"
    }
    
    # Check for MSI installer
    $msiPath = Join-Path $BuildPath "msi"
    if (Test-Path $msiPath) {
        $msiFiles = Get-ChildItem $msiPath -Filter "*.msi" -ErrorAction SilentlyContinue
        if ($msiFiles) {
            Write-Status "success" "MSI installer found: $($msiFiles[0].Name)"
            
            # Check file size
            $sizeKB = [math]::Round($msiFiles[0].Length / 1KB, 1)
            Write-Host "    📏 Size: $sizeKB KB" -ForegroundColor Gray
        } else {
            Write-Status "warning" "No MSI installer found in $msiPath"
        }
    } else {
        Write-Status "info" "MSI build directory not found (may not be configured)"
    }
} else {
    Write-Status "error" "Build directory not found: $BuildPath"
    Write-Status "info" "Run 'tauri build' to create Windows builds"
}
Write-Host ""

# Check Tauri configuration
Write-Host "⚙️  Checking Tauri configuration:" -ForegroundColor White
if (Test-Path "tauri.conf.json") {
    $config = Get-Content "tauri.conf.json" | ConvertFrom-Json
    
    # Check WebView2 configuration
    if ($config.bundle.windows.webviewInstallMode) {
        Write-Status "success" "WebView2 install mode configured: $($config.bundle.windows.webviewInstallMode.type)"
    } else {
        Write-Status "warning" "WebView2 install mode not configured"
        Write-Status "info" "Users may need to install WebView2 manually"
    }
    
    # Check code signing configuration
    if ($config.bundle.windows.certificateThumbprint) {
        Write-Status "success" "Code signing certificate configured"
    } else {
        Write-Status "warning" "No code signing certificate configured"
        Write-Status "info" "Builds will be unsigned and trigger SmartScreen warnings"
    }
    
    # Check icon configuration
    if ($config.bundle.icon -and ($config.bundle.icon -contains "icons/icon.ico")) {
        Write-Status "success" "Windows icon configured: icons/icon.ico"
        
        # Verify icon file exists
        if (Test-Path "icons/icon.ico") {
            $iconSize = [math]::Round((Get-Item "icons/icon.ico").Length / 1KB, 1)
            Write-Host "    📏 Icon size: $iconSize KB" -ForegroundColor Gray
        } else {
            Write-Status "error" "Icon file not found: icons/icon.ico"
        }
    } else {
        Write-Status "warning" "Windows icon not configured"
    }
} else {
    Write-Status "error" "tauri.conf.json not found"
}
Write-Host ""

# Check Windows Defender status
Write-Host "🛡️  Checking Windows Defender:" -ForegroundColor White
try {
    $defenderStatus = Get-MpComputerStatus -ErrorAction SilentlyContinue
    if ($defenderStatus) {
        if ($defenderStatus.RealTimeProtectionEnabled) {
            Write-Status "info" "Windows Defender real-time protection enabled"
            Write-Status "warning" "Unsigned executables may trigger warnings"
        } else {
            Write-Status "info" "Windows Defender real-time protection disabled"
        }
    } else {
        Write-Status "info" "Could not determine Windows Defender status"
    }
} catch {
    Write-Status "info" "Windows Defender status check not available"
}
Write-Host ""

# Summary and recommendations
Write-Host "📊 Summary and Recommendations:" -ForegroundColor White
Write-Host ""

$issues = @()
$recommendations = @()

if ($winVersion -lt $minVersion) {
    $issues += "Windows version compatibility"
    $recommendations += "Test on older Windows versions"
}

if (-not (Get-AppxPackage -Name "Microsoft.WebView2" -ErrorAction SilentlyContinue)) {
    $issues += "WebView2 Runtime not installed"
    $recommendations += "Ensure WebView2 auto-installation is configured"
}

if (Test-Path "tauri.conf.json") {
    $config = Get-Content "tauri.conf.json" | ConvertFrom-Json
    if (-not $config.bundle.windows.certificateThumbprint) {
        $issues += "No code signing certificate"
        $recommendations += "Purchase and configure Windows code signing certificate"
    }
}

if ($issues.Count -eq 0) {
    Write-Status "success" "No critical issues found"
} else {
    Write-Host "Issues found:" -ForegroundColor Yellow
    foreach ($issue in $issues) {
        Write-Host "  • $issue" -ForegroundColor Yellow
    }
}

if ($recommendations.Count -gt 0) {
    Write-Host ""
    Write-Host "Recommendations:" -ForegroundColor Cyan
    foreach ($rec in $recommendations) {
        Write-Host "  • $rec" -ForegroundColor Cyan
    }
}

Write-Host ""
Write-Host "🚀 Next steps:" -ForegroundColor White
Write-Host "   1. Purchase Windows code signing certificate" -ForegroundColor Gray
Write-Host "   2. Configure certificate in GitHub Secrets" -ForegroundColor Gray
Write-Host "   3. Test builds on clean Windows systems" -ForegroundColor Gray
Write-Host "   4. Verify WebView2 automatic installation" -ForegroundColor Gray
Write-Host ""

Write-Status "info" "Windows build verification completed!"
