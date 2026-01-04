class HapticHarmonySimulator < Formula
  desc "Haptic Harmony Ring Simulator for developers"
  homepage "https://github.com/gestura-ai/haptic-harmony-simulation"
  version "0.1.0"
  
  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/gestura-ai/haptic-harmony-simulation/releases/download/v#{version}/haptic-harmony-simulation-macos-arm64.dmg"
      sha256 "PLACEHOLDER_ARM64_SHA256"
    else
      url "https://github.com/gestura-ai/haptic-harmony-simulation/releases/download/v#{version}/haptic-harmony-simulation-macos-x64.dmg"
      sha256 "PLACEHOLDER_X64_SHA256"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/gestura-ai/haptic-harmony-simulation/releases/download/v#{version}/haptic-harmony-simulation-linux-arm64.tar.gz"
      sha256 "PLACEHOLDER_LINUX_ARM64_SHA256"
    else
      url "https://github.com/gestura-ai/haptic-harmony-simulation/releases/download/v#{version}/haptic-harmony-simulation-linux-x64.tar.gz"
      sha256 "PLACEHOLDER_LINUX_X64_SHA256"
    end
  end

  depends_on "rust" => :build if build.from_source?

  def install
    if OS.mac?
      # Extract from DMG and install
      system "hdiutil", "attach", cached_download
      app_path = "/Volumes/Haptic Harmony Simulator/Haptic Harmony Simulator.app"
      prefix.install app_path
      system "hdiutil", "detach", "/Volumes/Haptic Harmony Simulator"
      
      # Create symlink for CLI access
      bin.install_symlink "#{prefix}/Haptic Harmony Simulator.app/Contents/MacOS/haptic-harmony-simulation"
    else
      # Linux installation
      bin.install "haptic-harmony-simulation"
      
      # Install desktop file and icons if available
      if File.exist?("packaging/flatpak/ai.gestura.HapticHarmonySimulator.desktop")
        (share/"applications").install "packaging/flatpak/ai.gestura.HapticHarmonySimulator.desktop"
      end
      
      # Install icons
      %w[16 32 64 128 256 512].each do |size|
        icon_file = "icons/#{size}x#{size}.png"
        if File.exist?(icon_file)
          (share/"icons/hicolor/#{size}x#{size}/apps").install icon_file => "haptic-harmony-simulator.png"
        end
      end
    end
  end

  def caveats
    if OS.mac?
      <<~EOS
        The Haptic Harmony Simulator has been installed as an application.
        You can run it from Applications or use the command line:
          haptic-harmony-simulation --mode gui
          haptic-harmony-simulation --mode cli
      EOS
    else
      <<~EOS
        The Haptic Harmony Simulator has been installed.
        Run it with:
          haptic-harmony-simulation --mode gui
          haptic-harmony-simulation --mode cli
      EOS
    end
  end

  test do
    # Test that the binary exists and shows help
    assert_match "Haptic Harmony", shell_output("#{bin}/haptic-harmony-simulation --help")
  end
end
