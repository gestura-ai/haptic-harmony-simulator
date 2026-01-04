# macOS Code Signing and Notarization Setup

This guide explains how to set up automated code signing and notarization for the Haptic Harmony Simulator macOS builds in GitHub Actions.

## Prerequisites

### 1. Apple Developer Account
- **Paid Apple Developer Program membership** ($99/year)
- Access to [Apple Developer Portal](https://developer.apple.com)

### 2. Required Certificates
You need a **Developer ID Application Certificate** for distribution outside the Mac App Store.

## Step-by-Step Setup

### 1. Create and Download Certificate

1. **Log in to Apple Developer Portal**
   - Go to [developer.apple.com](https://developer.apple.com)
   - Sign in with your Apple ID

2. **Create Developer ID Application Certificate**
   - Navigate to **Certificates, Identifiers & Profiles**
   - Click **Certificates** → **+** (Add)
   - Select **Developer ID Application**
   - Follow the prompts to create a Certificate Signing Request (CSR)
   - Download the certificate (.cer file)

3. **Export Certificate as P12**
   - Double-click the .cer file to add it to Keychain Access
   - Open **Keychain Access** → **My Certificates**
   - Find your "Developer ID Application" certificate
   - Right-click → **Export**
   - Choose **Personal Information Exchange (.p12)**
   - Set a strong password (you'll need this for GitHub Secrets)
   - Save the .p12 file

### 2. Get App-Specific Password

1. **Create App-Specific Password**
   - Go to [appleid.apple.com](https://appleid.apple.com)
   - Sign in → **App-Specific Passwords**
   - Click **+** → Generate password
   - Label it "GitHub Actions Notarization"
   - **Save this password** (you can't view it again)

### 3. Configure GitHub Secrets

Add these secrets to your GitHub repository (**Settings** → **Secrets and variables** → **Actions**):

| Secret Name | Description | Example Value |
|-------------|-------------|---------------|
| `APPLE_CERTIFICATE` | Base64-encoded .p12 certificate | `MIIK...` (base64 string) |
| `APPLE_CERTIFICATE_PASSWORD` | Password for .p12 file | `your-p12-password` |
| `APPLE_SIGNING_IDENTITY` | Certificate common name | `Developer ID Application: Your Name (TEAM123)` |
| `APPLE_ID` | Your Apple ID email | `your-email@example.com` |
| `APPLE_PASSWORD` | App-specific password | `abcd-efgh-ijkl-mnop` |
| `APPLE_TEAM_ID` | Your Apple Developer Team ID | `ABC123DEF4` |

### 4. Get Required Values

#### Base64 Encode Certificate
```bash
# On macOS/Linux:
base64 -i /path/to/your/certificate.p12 | pbcopy

# On Windows (PowerShell):
[Convert]::ToBase64String([IO.File]::ReadAllBytes("C:\path\to\certificate.p12")) | Set-Clipboard
```

#### Find Signing Identity
```bash
# List all code signing identities
security find-identity -v -p codesigning

# Look for something like:
# "Developer ID Application: Your Name (TEAM123)"
```

#### Find Team ID
- Go to [Apple Developer Portal](https://developer.apple.com)
- **Account** → **Membership Details**
- Your **Team ID** is displayed (e.g., `ABC123DEF4`)

## How It Works

### 1. Certificate Import
```yaml
- name: Import Apple Developer Certificate (macOS)
  if: matrix.platform == 'macos-latest'
  uses: apple-actions/import-codesign-certs@v2
  with:
    p12-file-base64: ${{ secrets.APPLE_CERTIFICATE }}
    p12-password: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
```

### 2. Environment Setup
The workflow sets up environment variables that Tauri uses for signing:
- `APPLE_CERTIFICATE` - Certificate data
- `APPLE_SIGNING_IDENTITY` - Identity to use for signing
- `APPLE_ID` / `APPLE_PASSWORD` - For notarization
- `APPLE_TEAM_ID` - Your developer team

### 3. Automatic Signing
Tauri automatically:
- Signs the .app bundle with your Developer ID
- Creates a signed .dmg installer
- Submits to Apple for notarization
- Staples the notarization ticket

### 4. Verification
The workflow verifies:
- Code signature validity
- Notarization status
- Gatekeeper compatibility

## Troubleshooting

### Common Issues

1. **"No signing identity found"**
   - Check `APPLE_SIGNING_IDENTITY` matches exactly
   - Verify certificate is properly imported

2. **"Notarization failed"**
   - Check `APPLE_ID` and `APPLE_PASSWORD` are correct
   - Ensure app-specific password is used (not regular password)
   - Verify `APPLE_TEAM_ID` is correct

3. **"Certificate expired"**
   - Developer ID certificates are valid for 5 years
   - Renew in Apple Developer Portal
   - Update GitHub Secrets with new certificate

### Debug Commands

```bash
# List available signing identities
security find-identity -v -p codesigning

# Verify app signature
codesign --verify --verbose=2 /path/to/app.app

# Check notarization status
spctl --assess --type open --context context:primary-signature --verbose=2 /path/to/app.dmg
```

## Security Best Practices

1. **Rotate Certificates**: Update certificates before expiration
2. **Secure Secrets**: Never commit certificates or passwords to git
3. **Limit Access**: Only give repository access to trusted team members
4. **Monitor Usage**: Check Apple Developer Portal for signing activity

## Expected Results

After successful setup:
- ✅ macOS users can download and run the app without warnings
- ✅ App passes Gatekeeper security checks
- ✅ No "unidentified developer" warnings
- ✅ Professional appearance and user trust
