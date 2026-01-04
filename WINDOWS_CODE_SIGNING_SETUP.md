# Windows Code Signing Setup

This guide explains how to set up automated code signing for Windows builds of the Haptic Harmony Simulator.

## Why Windows Code Signing Matters

- ✅ **Windows Defender SmartScreen** blocks unsigned executables
- ✅ **"Windows protected your PC"** warnings prevent installation
- ✅ **Professional appearance** and user trust
- ✅ **Enterprise deployment** requirements
- ✅ **Antivirus compatibility** - signed binaries are less likely to be flagged

## Prerequisites

### 1. Code Signing Certificate
You need a **Code Signing Certificate** from a trusted Certificate Authority (CA):

**Recommended Providers:**
- **Sectigo (formerly Comodo)** - $200-400/year
- **DigiCert** - $400-600/year  
- **GlobalSign** - $300-500/year
- **SSL.com** - $200-300/year

**Certificate Types:**
- **Standard Code Signing** - Basic signing, requires manual validation
- **EV Code Signing** - Extended validation, immediate SmartScreen reputation

## Step-by-Step Setup

### 1. Purchase and Download Certificate

1. **Purchase Certificate** from your chosen CA
2. **Complete validation process** (business verification)
3. **Download certificate** as .p12/.pfx file
4. **Set a strong password** for the certificate file

### 2. Configure GitHub Secrets

Add these secrets to your GitHub repository (**Settings** → **Secrets and variables** → **Actions**):

| Secret Name | Description | Example Value |
|-------------|-------------|---------------|
| `WINDOWS_CERTIFICATE` | Base64-encoded .p12/.pfx certificate | `MIIK...` (base64 string) |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password for certificate file | `your-cert-password` |

### 3. Get Base64 Certificate

#### On Windows (PowerShell):
```powershell
# Convert certificate to base64
$certBytes = [IO.File]::ReadAllBytes("C:\path\to\your\certificate.p12")
$base64Cert = [Convert]::ToBase64String($certBytes)
$base64Cert | Set-Clipboard
```

#### On macOS/Linux:
```bash
# Convert certificate to base64
base64 -i /path/to/your/certificate.p12 | pbcopy
```

## How It Works

### 1. Certificate Import
```yaml
- name: Setup Windows code signing environment
  if: matrix.platform == 'windows-latest'
  shell: powershell
  run: |
    # Decode and import certificate
    $certBytes = [Convert]::FromBase64String("${{ secrets.WINDOWS_CERTIFICATE }}")
    Import-PfxCertificate -FilePath $certPath -CertStoreLocation Cert:\CurrentUser\My
```

### 2. Automatic Signing
Tauri automatically signs:
- ✅ Main executable (.exe)
- ✅ MSI installer package
- ✅ All bundled DLLs and dependencies

### 3. Timestamping
The workflow uses Sectigo's timestamp server for long-term signature validity:
```json
"timestampUrl": "http://timestamp.sectigo.com"
```

## Alternative: Azure Code Signing

For enterprise environments, consider **Azure Code Signing**:

### Benefits:
- ✅ **Cloud-based** - no certificate files to manage
- ✅ **Hardware Security Module (HSM)** protection
- ✅ **Automatic key rotation**
- ✅ **Audit logging**

### Setup:
1. Create **Azure Key Vault**
2. Import certificate to Key Vault
3. Use **Azure CLI** or **REST API** for signing
4. Configure service principal authentication

## Troubleshooting

### Common Issues

1. **"Certificate not found"**
   - Verify base64 encoding is correct
   - Check certificate password
   - Ensure certificate is not expired

2. **"Timestamp server unavailable"**
   - Try alternative timestamp servers:
     - `http://timestamp.digicert.com`
     - `http://timestamp.globalsign.com/tsa/r6advanced1`
     - `http://tsa.ssl.com/ts`

3. **"SmartScreen still shows warnings"**
   - New certificates need to build reputation
   - Consider EV Code Signing for immediate reputation
   - Submit signed binaries to Microsoft for analysis

### Debug Commands

```powershell
# Check certificate in store
Get-ChildItem -Path Cert:\CurrentUser\My -CodeSigningCert

# Verify signature
Get-AuthenticodeSignature -FilePath "path\to\your\app.exe"

# Sign manually (for testing)
signtool sign /f certificate.p12 /p password /t http://timestamp.sectigo.com /v app.exe
```

## Security Best Practices

1. **Protect Certificate Files**
   - Never commit certificates to git
   - Use strong passwords
   - Store securely with limited access

2. **Monitor Certificate Expiration**
   - Set calendar reminders
   - Automate expiration alerts
   - Plan renewal process

3. **Validate Signatures**
   - Test signed binaries on clean Windows systems
   - Monitor SmartScreen reputation
   - Check timestamp validity

## Expected Results

After successful setup:
- ✅ Windows users can download and install without warnings
- ✅ No "Windows protected your PC" dialogs
- ✅ Professional appearance in Windows
- ✅ Enterprise deployment compatibility
- ✅ Improved antivirus compatibility

## Cost Considerations

**Annual Costs:**
- **Standard Code Signing**: $200-600/year
- **EV Code Signing**: $400-800/year
- **Azure Code Signing**: $50-200/month (depending on usage)

**ROI Benefits:**
- ✅ Increased user trust and adoption
- ✅ Reduced support tickets about security warnings
- ✅ Professional brand image
- ✅ Enterprise sales opportunities
