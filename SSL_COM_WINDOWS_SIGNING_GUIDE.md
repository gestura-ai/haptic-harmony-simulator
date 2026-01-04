# SSL.com Windows Code Signing Implementation Guide
## Complete Setup for Haptic Harmony Simulator

This guide provides step-by-step instructions for implementing SSL.com Windows code signing in the Haptic Harmony Simulator project.

---

## 🔐 **Phase 1: Certificate Preparation**

### **Step 1: Prepare SSL.com Certificate**

Your SSL.com certificate should be in `.p12` or `.pfx` format. Convert it to base64 for GitHub Secrets:

#### **On Windows (PowerShell):**
```powershell
# Navigate to certificate location
cd "C:\path\to\your\certificate"

# Convert to base64
$certPath = "ssl-com-certificate.p12"  # Your SSL.com certificate file
$certBytes = [IO.File]::ReadAllBytes($certPath)
$base64Cert = [Convert]::ToBase64String($certBytes)

# Copy to clipboard
$base64Cert | Set-Clipboard
Write-Host "✅ Certificate copied to clipboard"
Write-Host "📋 Certificate size: $($base64Cert.Length) characters"
```

#### **On macOS/Linux:**
```bash
# Convert certificate to base64
base64 -i ssl-com-certificate.p12 | pbcopy
echo "✅ Certificate copied to clipboard"
```

### **Step 2: Extract Certificate Information**

Get the exact subject name from your SSL.com certificate:

```powershell
# Load certificate to check subject name
$cert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2("ssl-com-certificate.p12", "your-password")
Write-Host "Certificate Subject: $($cert.Subject)"
Write-Host "Certificate Issuer: $($cert.Issuer)"
Write-Host "Valid From: $($cert.NotBefore)"
Write-Host "Valid Until: $($cert.NotAfter)"

# Extract organization name for GitHub Secrets
$subjectParts = $cert.Subject -split ','
$orgName = ($subjectParts | Where-Object { $_ -like "*O=*" }) -replace '.*O=([^,]+).*', '$1'
Write-Host "Organization Name: $orgName"
```

---

## 🔑 **Phase 2: GitHub Secrets Configuration**

### **Required GitHub Secrets**

Navigate to your repository: **Settings** → **Secrets and variables** → **Actions** → **New repository secret**

| Secret Name | Description | Example Value |
|-------------|-------------|---------------|
| `WINDOWS_CERTIFICATE` | Base64-encoded SSL.com certificate | `MIIK...` (long base64 string) |
| `WINDOWS_CERTIFICATE_PASSWORD` | Certificate password | `your-ssl-com-password` |
| `WINDOWS_SIGNING_SUBJECT_NAME` | Organization name from certificate | `Gestura AI LLC` |

### **Verification Commands**

Test your secrets configuration:

```bash
# Check if secrets are properly set (run locally)
echo "Checking GitHub Secrets configuration..."
echo "WINDOWS_CERTIFICATE: $(if [ -n "$WINDOWS_CERTIFICATE" ]; then echo "✅ Set"; else echo "❌ Missing"; fi)"
echo "WINDOWS_CERTIFICATE_PASSWORD: $(if [ -n "$WINDOWS_CERTIFICATE_PASSWORD" ]; then echo "✅ Set"; else echo "❌ Missing"; fi)"
echo "WINDOWS_SIGNING_SUBJECT_NAME: $(if [ -n "$WINDOWS_SIGNING_SUBJECT_NAME" ]; then echo "✅ Set ($WINDOWS_SIGNING_SUBJECT_NAME)"; else echo "❌ Missing"; fi)"
```

---

## ⚙️ **Phase 3: Configuration Updates**

### **Tauri Configuration (tauri.conf.json)**

The configuration has been updated with SSL.com-specific settings:

```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.ssl.com",
      "webviewInstallMode": {
        "type": "downloadBootstrapper",
        "silent": true
      }
    }
  }
}
```

**Key SSL.com-specific changes:**
- ✅ **Timestamp URL**: Uses SSL.com's timestamp server (`timestamp.ssl.com`)
- ✅ **WebView2 Integration**: Automatic WebView2 installation
- ✅ **Professional Installer**: Enhanced NSIS and WiX configuration

### **GitHub Actions Integration**

The release workflow now includes SSL.com-specific certificate handling:

```yaml
- name: Setup SSL.com Windows code signing environment
  if: matrix.platform == 'windows-latest'
  shell: powershell
  run: |
    Write-Host "🔐 SSL.com Windows code signing certificate detected"
    
    # Import SSL.com certificate
    $certPath = "$env:RUNNER_TEMP\ssl-com-certificate.p12"
    $certBytes = [Convert]::FromBase64String("${{ secrets.WINDOWS_CERTIFICATE }}")
    [IO.File]::WriteAllBytes($certPath, $certBytes)
    
    # Certificate validation and environment setup
    # ... (detailed implementation in workflow)
```

---

## 🧪 **Phase 4: Testing and Verification**

### **Local Testing Commands**

```bash
# Verify Windows build configuration
just verify-windows-build

# Build and test locally (requires Windows)
just build-macos-app  # On macOS for comparison
# On Windows: tauri build --features tauri-gui
```

### **GitHub Actions Testing**

1. **Create Test Release:**
   ```bash
   # Push a tag to trigger release workflow
   git tag v0.1.1-test
   git push origin v0.1.1-test
   ```

2. **Monitor Build Process:**
   - Check GitHub Actions logs for SSL.com certificate import
   - Verify signature validation steps
   - Confirm both NSIS and MSI installers are created

3. **Download and Test:**
   - Download signed executables from GitHub Releases
   - Test on clean Windows 10/11 systems
   - Verify no SmartScreen warnings appear

### **SmartScreen Compatibility Testing**

#### **Test Scenarios:**
1. **Clean Windows 10 VM**: Test first-time download and execution
2. **Clean Windows 11 VM**: Test on latest Windows version
3. **Corporate Environment**: Test with enterprise security policies
4. **Different Browsers**: Test downloads from Chrome, Edge, Firefox

#### **Expected Results with SSL.com Certificate:**
- ✅ **No SmartScreen warnings** (with established certificate reputation)
- ✅ **Clean antivirus scans** (signed executables have fewer false positives)
- ✅ **Professional appearance** in Windows security dialogs
- ✅ **Enterprise deployment ready**

---

## 🔍 **Phase 5: Verification Checklist**

### **Pre-Release Checklist**

- [ ] SSL.com certificate imported successfully in GitHub Actions
- [ ] Certificate subject name matches `WINDOWS_SIGNING_SUBJECT_NAME` secret
- [ ] Timestamp server (`timestamp.ssl.com`) is accessible
- [ ] Both NSIS (.exe) and MSI installers are created
- [ ] Digital signatures are valid on both installer types
- [ ] WebView2 automatic installation works
- [ ] Icons are properly embedded in executables
- [ ] No SmartScreen warnings on test systems

### **Post-Release Verification**

```powershell
# Download and verify signed executable
$exePath = "haptic-harmony-simulator-setup.exe"
$signature = Get-AuthenticodeSignature -FilePath $exePath

Write-Host "Signature Status: $($signature.Status)"
Write-Host "Signer: $($signature.SignerCertificate.Subject)"
Write-Host "Timestamp: $($signature.TimeStamperCertificate.Subject)"

# Should show:
# Signature Status: Valid
# Signer: CN=Gestura AI LLC, O=Gestura AI LLC, ...
# Timestamp: CN=SSL.com Timestamp Server, ...
```

---

## 🌍 **Cross-Platform Considerations**

### **SSL.com Certificate Scope**

**Windows Code Signing Certificate:**
- ✅ **Windows executables**: .exe, .msi, .dll files
- ❌ **Linux packages**: Not applicable for DEB, AppImage, Snap
- ❌ **macOS applications**: Requires separate Apple Developer certificate

### **Linux Package Signing Alternatives**

Since SSL.com Windows certificates don't apply to Linux, consider these options:

#### **Option 1: GPG Signing (Recommended)**
```bash
# Create GPG key for package signing
gpg --full-generate-key
gpg --export --armor your-email@gestura.ai > gestura-public.key

# Sign DEB packages
dpkg-sig --sign builder package.deb

# Sign AppImages
gpg --detach-sign --armor package.AppImage
```

#### **Option 2: Repository Signing**
- **APT Repository**: Sign with GPG for Debian/Ubuntu
- **Snap Store**: Automatic signing by Canonical
- **Flatpak**: Sign with GPG for Flathub

#### **Option 3: Checksum Verification**
```bash
# Generate checksums for Linux packages
sha256sum *.deb *.AppImage > SHA256SUMS
gpg --clearsign SHA256SUMS
```

---

## 📊 **Monitoring and Maintenance**

### **Certificate Monitoring**

```powershell
# Check certificate expiration (run monthly)
$cert = Get-ChildItem -Path Cert:\CurrentUser\My -CodeSigningCert | Where-Object { $_.Subject -like "*Gestura*" }
$daysUntilExpiry = ($cert.NotAfter - (Get-Date)).Days
Write-Host "Certificate expires in $daysUntilExpiry days"

if ($daysUntilExpiry -lt 30) {
    Write-Host "⚠️ Certificate expires soon - plan renewal"
}
```

### **SmartScreen Reputation Building**

**SSL.com certificates provide:**
- ✅ **Immediate trust** (no reputation building period)
- ✅ **Extended Validation** benefits (if EV certificate)
- ✅ **Reduced false positives** in antivirus software

### **Renewal Process**

1. **30 days before expiration**: Purchase renewal from SSL.com
2. **Update GitHub Secrets** with new certificate
3. **Test signing process** with new certificate
4. **Update documentation** with new expiration date

---

## 🚀 **Implementation Timeline**

### **Immediate (Day 1)**
- [ ] Convert SSL.com certificate to base64
- [ ] Configure GitHub Secrets
- [ ] Test certificate import locally

### **Week 1**
- [ ] Complete GitHub Actions integration
- [ ] Test signed builds on Windows VMs
- [ ] Verify SmartScreen compatibility
- [ ] Document troubleshooting procedures

### **Week 2**
- [ ] Production release with signed executables
- [ ] Monitor user feedback and support tickets
- [ ] Set up certificate expiration monitoring
- [ ] Plan Linux package signing strategy

---

## 📋 **Success Metrics**

### **Technical Metrics**
- ✅ **100% signed releases** (no unsigned executables)
- ✅ **0% SmartScreen warnings** on signed builds
- ✅ **<5% antivirus false positives** (down from ~20% unsigned)
- ✅ **Certificate validation success rate**: 100%

### **User Experience Metrics**
- ✅ **Reduced support tickets** related to security warnings
- ✅ **Faster adoption** in enterprise environments
- ✅ **Professional brand perception**
- ✅ **Compliance with security policies**

---

**Status**: 🚀 **Ready for Implementation**  
**Priority**: 🔥 **Critical - Production Requirement**  
**Timeline**: Complete within 1 week  
**Next Action**: Configure GitHub Secrets with SSL.com certificate
