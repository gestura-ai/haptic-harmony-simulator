// Haptic Harmony Ring Simulator - Frontend Application
class RingSimulator {
    constructor() {
        this.isConnected = false;
        this.isRealRingConnected = false;
        this.batteryLevel = 85;
        this.isCharging = false;
        this.currentGesture = 'Ready';
        this.gestureConfidence = 100;
        this.logEntries = [];
        this.currentRingType = 'b1';
        this.currentInterface = 'ring';
        this.realRingDevice = null;
        this.tiltModeActive = false;
        this.tapCount = 0;
        this.lastTapTime = null;
        this.holdTimer = null;
        this.consoleVisible = false;
        this.previousWindowSize = null;

        // Slide tracking state
        this.isSliding = false;
        this.slideStartPos = null;
        this.slideCurrentPos = null;

        // Ring specifications
        this.ringSpecs = {
            b1: {
                name: 'B1 Haptic Harmony Ring',
                hapticZones: 4,
                features: ['Basic Gestures', 'Haptic Feedback', 'BLE Connectivity'],
                firmware: 'v1.2.3',
                maxIntensity: 100,
                patterns: ['pulse', 'wave', 'burst']
            },
            a1: {
                name: 'A1 Advanced Ring',
                hapticZones: 8,
                features: ['Advanced Gestures', 'Multi-Zone Haptics', 'AI Processing'],
                firmware: 'v2.0.0',
                maxIntensity: 150,
                patterns: ['pulse', 'wave', 'burst', 'cascade', 'spiral']
            },
            p1: {
                name: 'P1 Pro Ring',
                hapticZones: 12,
                features: ['Pro Gestures', 'Precision Haptics', 'Neural Interface'],
                firmware: 'v3.0.0',
                maxIntensity: 200,
                patterns: ['pulse', 'wave', 'burst', 'cascade', 'spiral', 'neural', 'adaptive']
            }
        };

        this.init();
    }

    init() {
        console.log('🚀 Initializing Ring Simulator...');
        this.setupEventListeners();
        this.setupInterfaceSwitching();
        this.setupConfigNavigation();
        this.setupRealRingConnection();
        this.updateRingDisplay();
        this.updateUI();
        this.updateVibrateButtonVisibility();
        this.initializeConsole(); // Initialize console as collapsed
        this.initializeInterfaceState(); // Initialize interface state
        this.updateConnectionStatus(); // Set initial status badge state
        this.startStatusUpdates();
        this.addLogEntry('info', 'Simulator initialized');
        console.log('✅ Ring Simulator initialization complete');
    }

    initializeInterfaceState() {
        // Initialize interface state - start with simulator mode
        const floatingControls = document.getElementById('floating-controls');

        // Show elements for simulator mode (default)
        if (floatingControls) floatingControls.style.display = 'flex';

        console.log('✅ Interface state initialized');
    }

    initializeConsole() {
        console.log('🔧 Initializing console...');

        // Start with console completely hidden
        const consoleSection = document.getElementById('console-section');
        const consoleToggle = document.getElementById('console-toggle');
        const collapseIcon = document.querySelector('#console-collapse .chevron-icon');

        if (consoleSection) {
            consoleSection.classList.add('collapsed');
            consoleSection.style.display = 'none'; // Completely hide initially
            console.log('✅ Console set to collapsed and hidden');
        }

        if (consoleToggle) {
            consoleToggle.classList.remove('active');
        }

        if (collapseIcon) {
            collapseIcon.textContent = '⬆️';
        }

        this.consoleVisible = false;
        console.log('✅ Console initialization complete');
    }

    setupEventListeners() {
        // Ring type selector
        const ringSelector = document.getElementById('ringType');
        if (ringSelector) {
            ringSelector.addEventListener('change', (e) => {
                this.switchRingType(e.target.value);
            });
        }

        // Gesture buttons
        document.querySelectorAll('.gesture-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const gesture = e.currentTarget.dataset.gesture;
                this.triggerGesture(gesture);
            });
        });

        // Haptic buttons
        document.querySelectorAll('.haptic-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const pattern = e.currentTarget.dataset.pattern;
                this.triggerHaptic(pattern);
            });
        });

        // Battery controls (if exists)
        const chargingToggle = document.getElementById('charging-toggle');
        if (chargingToggle) {
            chargingToggle.addEventListener('click', () => {
                this.toggleCharging();
            });
        }

        document.querySelectorAll('.preset-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const level = parseInt(e.currentTarget.dataset.level);
                this.setBatteryLevel(level);
            });
        });

        // Control buttons
        document.querySelectorAll('.control-btn').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const gesture = e.currentTarget.dataset.gesture;
                const action = e.currentTarget.dataset.action;

                if (gesture) {
                    this.triggerGesture(gesture);
                } else if (action === 'vibrate') {
                    this.triggerVibration();
                } else if (action === 'tilt-toggle') {
                    this.toggleTiltMode();
                }
            });
        });

        // Special handling for Tap button (can do tap, double-tap, hold)
        const tapBtn = document.getElementById('tap-btn');
        if (tapBtn) {
            let tapCount = 0;
            let tapTimer = null;

            tapBtn.addEventListener('click', (e) => {
                e.stopPropagation(); // Prevent double handling
                tapCount++;

                if (tapCount === 1) {
                    tapTimer = setTimeout(() => {
                        // Single tap
                        this.triggerGesture('tap');
                        tapCount = 0;
                    }, 300);
                } else if (tapCount === 2) {
                    // Double tap
                    clearTimeout(tapTimer);
                    this.triggerGesture('double-tap');
                    tapCount = 0;
                }
            });

            // Hold gesture on mousedown/mouseup
            let holdTimer = null;
            tapBtn.addEventListener('mousedown', () => {
                holdTimer = setTimeout(() => {
                    this.triggerGesture('hold');
                    tapCount = 0;
                    if (tapTimer) clearTimeout(tapTimer);
                }, 800);
            });

            tapBtn.addEventListener('mouseup', () => {
                if (holdTimer) {
                    clearTimeout(holdTimer);
                    holdTimer = null;
                }
            });
        }

        // SVG container interaction (tilt - anywhere in SVG area)
        const svgContainer = document.getElementById('hapticharmony-b1-shape');
        if (svgContainer) {
            console.log('Setting up SVG container events for tilt');
            // Add mouse move for 3D tilt tracking anywhere in SVG
            svgContainer.addEventListener('mousemove', (e) => {
                if (this.tiltModeActive) {
                    this.updateTiltDirection(e);
                }
            });

            svgContainer.addEventListener('mouseleave', () => {
                if (this.tiltModeActive) {
                    this.resetTiltDirection();
                }
            });
        } else {
            console.error('SVG container not found!');
        }

        // PAD SVG section interaction (tap/double-tap/hold/slide)
        const padSection = document.getElementById('pad');
        if (padSection) {
            console.log('Setting up PAD SVG section events');
            padSection.addEventListener('mousedown', (e) => {
                console.log('PAD SVG mousedown');
                e.stopPropagation(); // Prevent ring handler
                this.handlePadMouseDown(e);
            });
            padSection.addEventListener('mousemove', (e) => {
                // Always handle mouse move if we have a slide start position
                this.handlePadMouseMove(e);
            });
            padSection.addEventListener('mouseup', (e) => {
                console.log('PAD SVG mouseup');
                this.handlePadMouseUp(e);
            });
            padSection.addEventListener('mouseleave', (e) => {
                console.log('PAD SVG mouseleave');
                this.handlePadMouseUp(e); // Treat mouse leave as mouse up
            });
        } else {
            console.error('PAD SVG section not found!');
        }

        // Tilt toggle (center button)
        const tiltToggle = document.getElementById('tilt-toggle');
        if (tiltToggle) {
            tiltToggle.addEventListener('click', () => {
                this.toggleTiltMode();
            });
        }

        // Console toggle
        const consoleToggle = document.getElementById('console-toggle');
        const consoleHeader = document.getElementById('console-header');
        const consoleCollapse = document.getElementById('console-collapse');

        if (consoleToggle) {
            consoleToggle.addEventListener('click', () => {
                this.toggleConsole();
            });
        }

        if (consoleHeader) {
            consoleHeader.addEventListener('click', () => {
                this.toggleConsole();
            });
        }

        if (consoleCollapse) {
            consoleCollapse.addEventListener('click', (e) => {
                e.stopPropagation();
                this.toggleConsole();
            });
        }

        // Console resize functionality
        this.setupConsoleResize();

        // Global click handler for tilt mode release
        document.addEventListener('click', (e) => {
            if (this.tiltModeActive) {
                // Check if click is on ring or PAD area for tilt direction
                const ringElement = document.getElementById('ring');
                const padElement = document.getElementById('pad');
                const tiltToggle = document.getElementById('tilt-toggle');

                if (e.target === ringElement || e.target === padElement) {
                    // Handle tilt direction detection
                    this.handleTiltClick(e);
                } else if (e.target !== tiltToggle && !tiltToggle.contains(e.target)) {
                    // Click anywhere else releases tilt mode
                    console.log('🎯 Tilt mode released by clicking outside');
                    this.toggleTiltMode();
                }
            }
        });

        // Ring switching and modular functionality
    }

    setupInterfaceSwitching() {
        console.log('Setting up interface switching...');

        // Interface tab switching (only for ring/config, not console)
        const tabButtons = document.querySelectorAll('.interface-toggle-btn');
        console.log('Found tab buttons:', tabButtons.length);

        tabButtons.forEach(btn => {
            console.log('Setting up tab button:', btn.dataset.interface);
            btn.addEventListener('click', (e) => {
                e.preventDefault();
                const interfaceType = e.currentTarget.dataset.interface;
                console.log('Tab clicked:', interfaceType);
                if (interfaceType) {
                    this.switchInterface(interfaceType);
                }
            });
        });
    }

    setupConfigNavigation() {
        console.log('Setting up configuration navigation...');

        // Get all navigation links and sections
        const navLinks = document.querySelectorAll('.toc-link');
        const configSections = document.querySelectorAll('.config-section');

        console.log('Found nav links:', navLinks.length);
        console.log('Found config sections:', configSections.length);

        // Hide all sections initially except the first one (Ring Model)
        configSections.forEach((section, index) => {
            if (index === 0) {
                section.style.display = 'block';
            } else {
                section.style.display = 'none';
            }
        });

        // Add click handlers to navigation links
        navLinks.forEach(link => {
            link.addEventListener('click', (e) => {
                e.preventDefault();

                // Remove active class from all links
                navLinks.forEach(l => l.classList.remove('active'));

                // Add active class to clicked link
                e.target.classList.add('active');

                // Get target section ID from href
                const targetId = e.target.getAttribute('href').substring(1);
                console.log('Switching to config section:', targetId);

                // Hide all sections
                configSections.forEach(section => {
                    section.style.display = 'none';
                });

                // Show target section
                const targetSection = document.getElementById(targetId);
                if (targetSection) {
                    targetSection.style.display = 'block';
                    console.log('Showed section:', targetId);
                } else {
                    console.error('Section not found:', targetId);
                }
            });
        });
    }

    setupRealRingConnection() {
        // Scan for real ring button
        const scanBtn = document.getElementById('scan-btn');
        if (scanBtn) {
            scanBtn.addEventListener('click', () => {
                this.scanForRealRing();
            });
        }

        // Disconnect button
        const disconnectBtn = document.getElementById('disconnect-btn');
        if (disconnectBtn) {
            disconnectBtn.addEventListener('click', () => {
                this.disconnectRealRing();
            });
        }
    }

    async switchInterface(interfaceType) {
        console.log('🔄 Switching to interface:', interfaceType);

        // Handle window resizing with smooth animation
        if (window.__TAURI__ && window.__TAURI__.invoke) {
            try {
                if (interfaceType === 'config') {
                    // Store current window size before expanding (use default if not stored)
                    if (!this.previousWindowSize) {
                        this.previousWindowSize = { width: 800, height: 640 }; // Default size
                    }
                    console.log('📐 Storing window size:', this.previousWindowSize);

                    // Smooth resize to 800px width for config mode
                    await this.smoothWindowResize(this.previousWindowSize.width, 800, this.previousWindowSize.height);
                    console.log('📐 Expanded window to 800px width');
                } else if (interfaceType === 'ring' && this.previousWindowSize) {
                    // Smooth return to previous size when going back to simulator
                    await this.smoothWindowResize(800, this.previousWindowSize.width, this.previousWindowSize.height);
                    console.log('📐 Restored window to previous size:', this.previousWindowSize);
                }
            } catch (error) {
                console.error('📐 Failed to resize window:', error);
            }
        } else if (window.__TAURI__) {
            console.log('📐 Tauri API partially available, skipping window resize');
        } else {
            console.log('📐 Running in browser mode, window resize disabled');
        }

        // Hide/show status badge based on interface
        const statusBadge = document.getElementById('status-badge-corner');
        if (statusBadge) {
            if (interfaceType === 'config') {
                statusBadge.style.display = 'none';
            } else {
                statusBadge.style.display = 'block';
            }
        }

        // Hide all interface panels
        const panels = document.querySelectorAll('.interface-panel');
        console.log('Found panels:', panels.length);
        panels.forEach(panel => {
            panel.classList.remove('active');
            console.log('Removed active from:', panel.id);
        });

        // Remove active class from all tabs
        const tabs = document.querySelectorAll('.interface-toggle-btn');
        console.log('Found tabs:', tabs.length);
        tabs.forEach(btn => {
            btn.classList.remove('active');
            console.log('Removed active from tab:', btn.dataset.interface);
        });

        // Show selected interface
        const targetPanel = document.getElementById(`${interfaceType}-interface`);
        console.log('Target panel:', targetPanel);
        if (targetPanel) {
            targetPanel.classList.add('active');
            console.log('✅ Activated panel:', targetPanel.id);
        } else {
            console.error('❌ Target panel not found:', `${interfaceType}-interface`);
        }

        // Activate selected tab
        const targetTab = document.querySelector(`[data-interface="${interfaceType}"]`);
        console.log('Target tab:', targetTab);
        if (targetTab) {
            targetTab.classList.add('active');
            console.log('✅ Activated tab:', interfaceType);
        } else {
            console.error('❌ Target tab not found for interface:', interfaceType);
        }

        // Hide/show floating controls based on interface
        const floatingControls = document.getElementById('floating-controls');

        console.log('🔄 Managing interface state for:', interfaceType);

        if (interfaceType === 'config') {
            // Hide floating controls in config mode
            if (floatingControls) {
                floatingControls.style.display = 'none';
                console.log('✅ Hidden floating controls');
            }
        } else {
            // Show floating controls in simulator mode
            if (floatingControls) {
                floatingControls.style.display = 'flex';
                console.log('✅ Shown floating controls');
            }
        }

        this.currentInterface = interfaceType;
        this.addLogEntry('info', `Switched to ${interfaceType} interface`);
    }

    async smoothWindowResize(fromWidth, toWidth, height) {
        if (!window.__TAURI__ || !window.__TAURI__.invoke || fromWidth === toWidth) {
            console.log('📐 Skipping window resize - Tauri API not available or no size change needed');
            return;
        }

        console.log(`🎬 Starting smooth resize from ${fromWidth}px to ${toWidth}px`);

        try {
            const steps = 8; // Fewer steps for better performance
            const duration = 300; // Shorter duration
            const stepDelay = duration / steps;
            const widthDiff = toWidth - fromWidth;
            const stepSize = widthDiff / steps;

            for (let i = 1; i <= steps; i++) {
                const currentWidth = Math.round(fromWidth + (stepSize * i));

                try {
                    // Use the invoke method directly - this is more reliable
                    await window.__TAURI__.invoke('set_window_size', {
                        width: currentWidth,
                        height: height
                    });

                    console.log(`📐 Step ${i}/${steps}: Resized to ${currentWidth}x${height}`);

                    // Wait between steps for smooth animation
                    if (i < steps) {
                        await new Promise(resolve => setTimeout(resolve, stepDelay));
                    }
                } catch (error) {
                    console.error(`Failed to resize window at step ${i}:`, error);
                    break;
                }
            }

            console.log(`✅ Smooth resize completed to ${toWidth}px`);
        } catch (error) {
            console.error('Failed to perform smooth resize:', error);
            // Try direct resize as fallback
            await this.fallbackWindowResize(toWidth, height);
        }
    }

    async fallbackWindowResize(width, height) {
        try {
            await window.__TAURI__.invoke('set_window_size', { width, height });
            console.log(`📐 Fallback resize to ${width}x${height}`);
        } catch (error) {
            console.error('Fallback resize failed:', error);
        }
    }

    async scanForRealRing() {
        this.addLogEntry('info', 'Scanning for real Haptic Harmony Ring...');

        try {
            // Use existing Tauri command to scan for rings
            if (window.__TAURI__) {
                const devices = await window.__TAURI__.invoke('scan_for_rings');

                if (devices && devices.length > 0) {
                    // Found real ring(s) - devices is array of device IDs
                    this.realRingDevice = { id: devices[0], name: 'Haptic Harmony Ring' };
                    await this.connectToRealRing(this.realRingDevice);
                } else {
                    this.addLogEntry('warning', 'No real Haptic Harmony Ring found');
                }
            }
        } catch (error) {
            this.addLogEntry('error', `Scan failed: ${error}`);
        }
    }

    async connectToRealRing(device) {
        try {
            this.addLogEntry('info', `Connecting to ${device.name}...`);

            if (window.__TAURI__) {
                // Use existing pair_ring command
                await window.__TAURI__.invoke('pair_ring', { device_id: device.id });

                // Start gesture monitoring
                await window.__TAURI__.invoke('start_gesture_monitoring', { device_id: device.id });

                this.isRealRingConnected = true;
                this.enableHapticControls();
                this.updateConnectionStatus();
                this.addLogEntry('success', `Connected to real ring: ${device.name}`);

                // Update disconnect button
                const disconnectBtn = document.getElementById('disconnect-btn');
                if (disconnectBtn) {
                    disconnectBtn.disabled = false;
                }
            }
        } catch (error) {
            this.addLogEntry('error', `Connection error: ${error}`);
        }
    }

    async disconnectRealRing() {
        if (this.isRealRingConnected && this.realRingDevice) {
            try {
                this.addLogEntry('info', 'Disconnecting from real ring...');

                if (window.__TAURI__) {
                    // Stop gesture monitoring
                    await window.__TAURI__.invoke('stop_gesture_monitoring', { device_id: this.realRingDevice.id });
                }

                this.isRealRingConnected = false;
                this.realRingDevice = null;
                this.disableHapticControls();
                this.updateConnectionStatus();
                this.addLogEntry('info', 'Disconnected from real ring');

                // Update disconnect button
                const disconnectBtn = document.getElementById('disconnect-btn');
                if (disconnectBtn) {
                    disconnectBtn.disabled = true;
                }
            } catch (error) {
                this.addLogEntry('error', `Disconnect error: ${error}`);
            }
        }
    }

    enableHapticControls() {
        // Show vibrate button in ring interface
        this.updateVibrateButtonVisibility();

        // Enable haptic buttons in config interface
        document.querySelectorAll('.haptic-btn').forEach(btn => {
            btn.disabled = false;
        });

        const intensitySlider = document.getElementById('intensity-slider');
        if (intensitySlider) {
            intensitySlider.disabled = false;
        }

        // Update haptic badge
        const hapticBadge = document.getElementById('haptic-badge');
        if (hapticBadge) {
            hapticBadge.textContent = 'Real Ring Connected';
            hapticBadge.style.background = 'var(--success-color)';
        }
    }

    disableHapticControls() {
        // Hide vibrate button in ring interface
        this.updateVibrateButtonVisibility();

        // Disable haptic buttons in config interface
        document.querySelectorAll('.haptic-btn').forEach(btn => {
            btn.disabled = true;
        });

        const intensitySlider = document.getElementById('intensity-slider');
        if (intensitySlider) {
            intensitySlider.disabled = true;
        }

        // Update haptic badge
        const hapticBadge = document.getElementById('haptic-badge');
        if (hapticBadge) {
            hapticBadge.textContent = 'Real Ring Required';
            hapticBadge.style.background = 'var(--warning-color)';
        }
    }

    updateConnectionStatus() {
        const statusText = document.getElementById('ring-status-text');
        const statusBadge = document.getElementById('status-badge-corner');
        const statusIcon = document.getElementById('ring-status-icon');
        const statusIndicator = document.querySelector('.status-indicator');
        const connectionMode = document.getElementById('connection-mode');
        const deviceName = document.getElementById('device-name');

        if (this.isRealRingConnected) {
            if (statusText) statusText.textContent = 'Diagnostic Mode';
            if (statusBadge) {
                statusBadge.classList.remove('unauthenticated');
                statusBadge.classList.add('authenticated');
            }
            if (statusIndicator) statusIndicator.classList.add('connected');
            if (connectionMode) connectionMode.textContent = 'Real Hardware';
            if (deviceName) deviceName.textContent = this.realRingDevice?.name || 'Haptic Harmony Ring';
        } else {
            if (statusText) statusText.textContent = 'Sim Mode';
            if (statusBadge) {
                statusBadge.classList.remove('authenticated');
                statusBadge.classList.add('unauthenticated');
            }
            if (statusIndicator) statusIndicator.classList.remove('connected');
            if (connectionMode) connectionMode.textContent = 'Simulation';
            if (deviceName) deviceName.textContent = 'Virtual Ring';
        }

        // Update vibrate button state
        this.updateVibrateButtonVisibility();
    }

    animateRingZone(zoneElement) {
        // Add visual feedback when ring zone is clicked
        zoneElement.classList.add('active');
        setTimeout(() => {
            zoneElement.classList.remove('active');
        }, 300);
    }



    triggerVibrate() {
        if (!this.isRealRingConnected) {
            this.addLogEntry('warning', 'Vibrate requires real ring connection');
            return;
        }

        this.addLogEntry('info', 'Vibrate triggered');
        this.triggerHaptic('pulse');
    }

    updateVibrateButtonVisibility() {
        const vibrateBtn = document.getElementById('vibrate-btn');
        if (vibrateBtn) {
            // Always show the button, but enable/disable based on connection
            vibrateBtn.style.display = 'inline-block';

            if (this.isRealRingConnected) {
                vibrateBtn.disabled = false;
                vibrateBtn.title = 'Send vibration to connected ring';
            } else {
                vibrateBtn.disabled = true;
                vibrateBtn.title = 'Connect to a real ring to enable vibration';
            }
        }
    }

    handlePadMouseDown(e) {
        const now = Date.now();

        // Show tap indicator immediately at click location
        this.showTapIndicator(e.clientX, e.clientY);

        // Initialize slide tracking
        this.isSliding = false;
        this.slideStartPos = { x: e.clientX, y: e.clientY };
        this.slideCurrentPos = { x: e.clientX, y: e.clientY };

        // Check for double tap
        if (this.lastTapTime && (now - this.lastTapTime) < 300) {
            this.tapCount++;
            if (this.tapCount >= 2) {
                // Double tap detected
                this.triggerGesture('double-tap');
                this.resetTapState();
                return;
            }
        } else {
            this.tapCount = 1;
        }

        this.lastTapTime = now;

        // Start hold timer (longer delay to avoid interfering with slide)
        this.holdTimer = setTimeout(() => {
            if (this.tapCount === 1 && !this.isSliding) {
                // Hold detected (only if not sliding)
                this.triggerGesture('hold');
                this.resetTapState();
            }
        }, 800); // 800ms for hold detection (longer to avoid slide interference)
    }

    handlePadMouseMove(e) {
        if (!this.slideStartPos) return;

        // Update current position
        this.slideCurrentPos = { x: e.clientX, y: e.clientY };

        // Calculate distance moved
        const deltaX = this.slideCurrentPos.x - this.slideStartPos.x;
        const deltaY = this.slideCurrentPos.y - this.slideStartPos.y;
        const distance = Math.sqrt(deltaX * deltaX + deltaY * deltaY);

        // Get PAD element dimensions to calculate half distance
        const padElement = document.getElementById('pad');
        if (padElement) {
            const rect = padElement.getBoundingClientRect();
            const padRadius = Math.min(rect.width, rect.height) / 2;
            const halfPadDistance = padRadius / 2; // Half the distance of the pad

            // If moved more than half the pad distance, start sliding
            if (distance > halfPadDistance && !this.isSliding) {
                this.isSliding = true;
                console.log(`🎯 Slide started - distance: ${distance.toFixed(1)}px (threshold: ${halfPadDistance.toFixed(1)}px)`);

                // CRITICAL: Clear hold timer since we're now sliding
                if (this.holdTimer) {
                    clearTimeout(this.holdTimer);
                    this.holdTimer = null;
                    console.log('🚫 Hold timer cancelled - slide detected');
                }
            }
            // If any movement detected (even small), clear hold timer to prevent hold trigger
            else if (distance > 5 && this.holdTimer) { // 5px threshold for any movement
                clearTimeout(this.holdTimer);
                this.holdTimer = null;
                console.log('🚫 Hold timer cancelled - movement detected');
            }
        }
    }

    handlePadMouseUp(e) {
        // Check if this was a slide gesture first
        if (this.isSliding) {
            this.triggerGesture('slide');
            this.resetTapState();
            this.resetSlideState();
            return;
        }

        // Clear hold timer if still active
        const holdTimerWasActive = !!this.holdTimer;
        if (this.holdTimer) {
            clearTimeout(this.holdTimer);
            this.holdTimer = null;
        }

        // Reset slide state
        this.resetSlideState();

        // If hold timer was still active (no movement detected), this could be a hold or tap
        // If hold timer was cancelled (movement detected), this is just a tap
        if (holdTimerWasActive) {
            // Hold timer was still running, so this was a quick release - treat as tap
            setTimeout(() => {
                if (this.tapCount === 1 && this.lastTapTime) {
                    this.triggerGesture('tap');
                    this.resetTapState();
                }
            }, 300); // Wait for potential double tap
        } else {
            // Hold timer was already cancelled due to movement, so this is just a tap
            setTimeout(() => {
                if (this.tapCount === 1 && this.lastTapTime) {
                    this.triggerGesture('tap');
                    this.resetTapState();
                }
            }, 300);
        }
    }

    isSlideGesture(e) {
        // Simple slide detection - if mouse moved out of PAD area, consider it a slide
        return e.type === 'mouseleave';
    }

    showTapIndicator(x = null, y = null) {
        const indicator = document.getElementById('tap-indicator');
        if (indicator) {
            // Position indicator at click location if coordinates provided
            if (x !== null && y !== null) {
                const ringVisual = document.getElementById('ring-visual');
                if (ringVisual) {
                    const rect = ringVisual.getBoundingClientRect();
                    const relativeX = x - rect.left;
                    const relativeY = y - rect.top;

                    indicator.style.left = `${relativeX}px`;
                    indicator.style.top = `${relativeY}px`;
                }
            }

            indicator.classList.add('active');
            setTimeout(() => {
                indicator.classList.remove('active');
            }, 600);
        }
    }

    resetTapState() {
        this.tapCount = 0;
        this.lastTapTime = null;
        if (this.holdTimer) {
            clearTimeout(this.holdTimer);
            this.holdTimer = null;
        }
    }

    resetSlideState() {
        this.isSliding = false;
        this.slideStartPos = null;
        this.slideCurrentPos = null;
    }

    toggleTiltMode() {
        console.log('🎯 Tilt toggle clicked, current state:', this.tiltModeActive);
        this.tiltModeActive = !this.tiltModeActive;

        const tiltToggle = document.getElementById('tilt-toggle');
        const ringVisual = document.getElementById('ring-visual');

        if (tiltToggle) {
            if (this.tiltModeActive) {
                tiltToggle.classList.add('tilt-active');
                if (ringVisual) ringVisual.classList.add('tilt-mode');
                // Tilt mode activated (no log entry)
                this.updateGestureStatus('Tilt Mode Active - Escape to disable');
                this.startTiltEscapePattern();
            } else {
                tiltToggle.classList.remove('tilt-active');
                if (ringVisual) ringVisual.classList.remove('tilt-mode');
                // Tilt mode deactivated (no log entry)
                this.updateGestureStatus('Ready');
                this.stopTiltEscapePattern();
            }
        }
    }

    startTiltEscapePattern() {
        // Start listening for escape pattern (3 quick taps)
        this.tiltEscapeCount = 0;
        this.tiltEscapeTimer = null;
        console.log('🎯 Tilt escape pattern listening started');
    }

    stopTiltEscapePattern() {
        if (this.tiltEscapeTimer) {
            clearTimeout(this.tiltEscapeTimer);
            this.tiltEscapeTimer = null;
        }
        this.tiltEscapeCount = 0;
        console.log('🎯 Tilt escape pattern stopped');
    }

    handleTiltClick(e) {
        console.log('🎯 Tilt click detected');

        // Get the SVG container for proper positioning
        const svg = document.getElementById('hapticharmony-b1-shape');
        if (!svg) return;

        const rect = svg.getBoundingClientRect();
        const centerX = rect.left + rect.width / 2;
        const centerY = rect.top + rect.height / 2;
        const clickX = e.clientX;
        const clickY = e.clientY;

        // Calculate angle in degrees (0° = top, clockwise)
        const angle = Math.atan2(clickY - centerY, clickX - centerX) * (180 / Math.PI);
        const normalizedAngle = ((angle + 90) % 360 + 360) % 360; // Normalize to 0-360°

        console.log(`🎯 Tilt direction: ${normalizedAngle.toFixed(1)}°`);
        this.addLogEntry('gesture', `TILT ${normalizedAngle.toFixed(1)}° - ${new Date().toLocaleTimeString()}`);

        // Cancel tilt mode after direction detection
        this.toggleTiltMode();
    }

    handleRingRimClick(e) {
        console.log('🎯 Ring rim clicked');
        // This method is kept for compatibility but tilt handling is now in handleTiltClick
    }

    updateTiltDirection(e) {
        if (!this.tiltModeActive) return;

        const rect = e.currentTarget.getBoundingClientRect();
        const centerX = rect.left + rect.width / 2;
        const centerY = rect.top + rect.height / 2;
        const mouseX = e.clientX;
        const mouseY = e.clientY;

        // Calculate tilt angles based on mouse position (inverted for correct effect)
        const deltaX = (mouseX - centerX) / (rect.width / 2);
        const deltaY = (mouseY - centerY) / (rect.height / 2);

        // Limit tilt angles to reasonable range (inverted: mouse side up, opposite down)
        const maxTilt = 20;
        const tiltX = Math.max(-maxTilt, Math.min(maxTilt, -deltaY * maxTilt)); // Inverted Y
        const tiltY = Math.max(-maxTilt, Math.min(maxTilt, deltaX * maxTilt)); // Inverted X

        // Apply 3D transform to ring visual
        const ringVisual = document.querySelector('.ring-visual');
        if (ringVisual) {
            ringVisual.style.transform = `perspective(800px) rotateX(${tiltX}deg) rotateY(${tiltY}deg)`;
        }
    }

    resetTiltDirection() {
        const ringVisual = document.querySelector('.ring-visual');
        if (ringVisual) {
            ringVisual.style.transform = 'perspective(800px) rotateX(0deg) rotateY(0deg)';
        }
    }

    handleTiltEscapePattern() {
        // Simplified - any ring rim click cancels tilt mode
        if (this.tiltModeActive) {
            console.log('🎯 Tilt mode cancelled by ring rim click');
            return true;
        }
        return false;
    }

    toggleConsole() {
        console.log('🔄 Toggle console called, current state:', this.consoleVisible);
        this.consoleVisible = !this.consoleVisible;
        console.log('🔄 New console state:', this.consoleVisible);

        const consoleSection = document.getElementById('console-section');
        const consoleToggle = document.getElementById('console-toggle');
        const collapseIcon = document.querySelector('#console-collapse .chevron-icon');

        console.log('🔍 Console elements found:', {
            consoleSection: !!consoleSection,
            consoleToggle: !!consoleToggle,
            collapseIcon: !!collapseIcon
        });

        if (consoleSection) {
            if (this.consoleVisible) {
                console.log('👁️ Showing console');
                consoleSection.style.display = 'block';
                consoleSection.classList.add('expanded');
                consoleSection.classList.remove('collapsed');
                consoleSection.style.height = '300px'; // Default expanded height
                if (consoleToggle) consoleToggle.classList.add('active');
                if (collapseIcon) collapseIcon.textContent = '⬇️';
            } else {
                console.log('🙈 Hiding console');
                consoleSection.style.display = 'none';
                consoleSection.classList.remove('expanded');
                consoleSection.classList.add('collapsed');
                if (consoleToggle) consoleToggle.classList.remove('active');
                if (collapseIcon) collapseIcon.textContent = '⬆️';
            }
        } else {
            console.error('❌ Console section not found!');
        }
    }

    updateGestureStatus(status) {
        const gestureStatus = document.getElementById('gesture-status');
        if (gestureStatus) {
            gestureStatus.textContent = status;
        }
    }

    setupConsoleResize() {
        const resizeHandle = document.getElementById('console-resize-handle');
        const consoleSection = document.getElementById('console-section');

        if (!resizeHandle || !consoleSection) return;

        let isResizing = false;
        let startY = 0;
        let startHeight = 0;

        resizeHandle.addEventListener('mousedown', (e) => {
            isResizing = true;
            startY = e.clientY;
            startHeight = parseInt(window.getComputedStyle(consoleSection).height, 10);
            document.body.style.cursor = 'ns-resize';
            e.preventDefault();
        });

        document.addEventListener('mousemove', (e) => {
            if (!isResizing) return;

            const deltaY = startY - e.clientY; // Inverted because we're resizing from bottom
            const newHeight = Math.max(40, Math.min(window.innerHeight * 0.6, startHeight + deltaY));

            consoleSection.style.height = `${newHeight}px`;

            // Show console content if height is more than header
            if (newHeight > 40) {
                consoleSection.classList.add('expanded');
                consoleSection.classList.remove('collapsed');
                this.consoleVisible = true;
            } else {
                consoleSection.classList.remove('expanded');
                consoleSection.classList.add('collapsed');
                this.consoleVisible = false;
            }
        });

        document.addEventListener('mouseup', () => {
            if (isResizing) {
                isResizing = false;
                document.body.style.cursor = '';
            }
        });
    }

    switchRingType(ringType) {
        if (this.ringSpecs[ringType]) {
            this.currentRingType = ringType;
            this.updateRingDisplay();
            this.addLogEntry('info', `Switched to ${this.ringSpecs[ringType].name}`);

            // Notify backend about ring type change
            if (window.__TAURI__) {
                try {
                    window.__TAURI__.invoke('switch_ring_type', { ring_type: ringType });
                } catch (error) {
                    this.addLogEntry('error', `Failed to switch ring type: ${error}`);
                }
            }
        }
    }

    updateRingDisplay() {
        const spec = this.ringSpecs[this.currentRingType];

        // Update ring name
        const nameElement = document.getElementById('ringModelName');
        if (nameElement) {
            nameElement.textContent = spec.name;
        }

        // Update specifications
        const specsElement = document.getElementById('ringSpecs');
        if (specsElement) {
            specsElement.innerHTML = `
                <div class="spec-item">
                    <span class="spec-label">Haptic Zones:</span>
                    <span class="spec-value">${spec.hapticZones}</span>
                </div>
                <div class="spec-item">
                    <span class="spec-label">Battery:</span>
                    <span class="spec-value" id="batteryLevel">${this.batteryLevel}%</span>
                </div>
                <div class="spec-item">
                    <span class="spec-label">Firmware:</span>
                    <span class="spec-value">${spec.firmware}</span>
                </div>
                <div class="spec-item">
                    <span class="spec-label">Max Intensity:</span>
                    <span class="spec-value">${spec.maxIntensity}%</span>
                </div>
            `;
        }

        // Update haptic zones visualization
        this.updateHapticZones();

        // Update available patterns
        this.updatePatternSelector();
    }

    updateHapticZones() {
        const spec = this.ringSpecs[this.currentRingType];
        const ringViz = document.getElementById('ring-visual');

        if (!ringViz) {
            console.warn('Ring visual not found, skipping haptic zones update');
            return;
        }

        const hapticIndicators = ringViz.querySelector('.haptic-indicators');

        if (hapticIndicators) {
            hapticIndicators.innerHTML = '';

            // Create haptic zones based on ring type
            for (let i = 0; i < spec.hapticZones; i++) {
                const angle = (360 / spec.hapticZones) * i;
                const dot = document.createElement('div');
                dot.className = 'haptic-dot';
                dot.dataset.zone = i + 1;
                dot.style.transform = `rotate(${angle}deg) translateY(-40px)`;
                hapticIndicators.appendChild(dot);
            }
        }
    }

    updatePatternSelector() {
        const spec = this.ringSpecs[this.currentRingType];
        const patternSelect = document.querySelector('.pattern-select');

        if (patternSelect) {
            patternSelect.innerHTML = '';
            spec.patterns.forEach(pattern => {
                const option = document.createElement('option');
                option.value = pattern;
                option.textContent = pattern.charAt(0).toUpperCase() + pattern.slice(1);
                patternSelect.appendChild(option);
            });
        }

        // Intensity slider
        const intensitySlider = document.getElementById('intensity-slider');
        const intensityValue = document.getElementById('intensity-value');
        
        intensitySlider.addEventListener('input', (e) => {
            const value = e.target.value;
            intensityValue.textContent = `${value}%`;
        });

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {
            this.handleKeyboard(e);
        });

        // Tauri event listeners (if available)
        if (window.__TAURI__) {
            this.setupTauriListeners();
        }
    }

    setupTauriListeners() {
        // Listen for events from Rust backend
        window.__TAURI__.event.listen('gesture-detected', (event) => {
            const { gesture, confidence } = event.payload;
            this.updateGestureDisplay(gesture, confidence);
        });

        window.__TAURI__.event.listen('haptic-command', (event) => {
            const { pattern, intensity, duration } = event.payload;
            this.displayHapticFeedback(pattern, intensity, duration);
        });

        window.__TAURI__.event.listen('battery-update', (event) => {
            const { level, charging, voltage, temperature, health } = event.payload;
            this.updateBatteryDisplay(level, charging, voltage, temperature, health);
        });

        window.__TAURI__.event.listen('ble-status', (event) => {
            const { status, connected_clients } = event.payload;
            this.updateBLEStatus(status, connected_clients);
        });

        window.__TAURI__.event.listen('log-message', (event) => {
            const { level, message } = event.payload;
            this.addLogEntry(level, message);
        });
    }

    handleKeyboard(e) {
        // Prevent default for our shortcuts
        const shortcuts = {
            'Enter': 'tap',
            ' ': 'hold',
            't': 'tilt',
            's': 'slide',
            'd': 'double-tap',
            'h': 'notify',
            'b': 'buzz',
            'p': 'pulse',
            'c': 'toggle-charging',
            '1': 'battery-10',
            '2': 'battery-50',
            '3': 'battery-90',
            'i': 'battery-info'
        };

        if (shortcuts[e.key]) {
            e.preventDefault();
            
            if (shortcuts[e.key].startsWith('battery-')) {
                const level = parseInt(shortcuts[e.key].split('-')[1]);
                this.setBatteryLevel(level);
            } else if (shortcuts[e.key] === 'toggle-charging') {
                this.toggleCharging();
            } else if (shortcuts[e.key] === 'battery-info') {
                this.showBatteryInfo();
            } else if (['notify', 'buzz', 'pulse'].includes(shortcuts[e.key])) {
                this.triggerHaptic(shortcuts[e.key]);
            } else {
                this.triggerGesture(shortcuts[e.key]);
            }
        }
    }

    async triggerGesture(gesture) {
        console.log(`🎯 Gesture triggered: ${gesture}`);

        // Add detailed console logging
        const timestamp = new Date().toLocaleTimeString();
        this.addLogEntry('gesture', `${gesture.toUpperCase()} - ${timestamp}`);

        // Update UI immediately
        this.updateGestureDisplay(gesture, Math.floor(Math.random() * 20) + 80);

        // Show visual feedback
        this.showTapIndicator();

        // Update gesture status
        this.updateGestureStatus(`${gesture} detected`);

        // Call Tauri backend if available
        if (window.__TAURI__) {
            try {
                await window.__TAURI__.invoke('trigger_gesture', { gesture });
                this.addLogEntry('info', `Sent ${gesture} to backend`);
            } catch (error) {
                this.addLogEntry('error', `Failed to trigger gesture: ${error}`);
            }
        }

        // Reset gesture after animation
        setTimeout(() => {
            this.updateGestureDisplay('Ready', 100);
            this.updateGestureStatus('Ready');
        }, 2000);
    }

    async triggerHaptic(pattern) {
        // Only allow haptic feedback with real ring connected
        if (!this.isRealRingConnected) {
            this.addLogEntry('warning', 'Haptic feedback requires real ring connection');
            return;
        }

        const intensitySlider = document.getElementById('intensity-slider');
        const intensity = intensitySlider ? intensitySlider.value / 100 : 0.8;

        this.addLogEntry('info', `Haptic triggered: ${pattern} (${Math.round(intensity * 100)}%)`);

        // Animate haptic indicators
        this.animateHaptic(pattern, intensity);

        // Send to real ring via backend
        if (window.__TAURI__ && this.realRingDevice) {
            try {
                await window.__TAURI__.invoke('send_haptic_feedback', {
                    device_id: this.realRingDevice.id,
                    pattern: pattern,
                    intensity: intensity,
                    duration_ms: 500
                });
            } catch (error) {
                this.addLogEntry('error', `Haptic trigger failed: ${error}`);
            }
        }
    }

    async toggleCharging() {
        this.isCharging = !this.isCharging;
        this.addLogEntry('info', `Charging ${this.isCharging ? 'started' : 'stopped'}`);
        
        // Update UI
        this.updateBatteryDisplay(this.batteryLevel, this.isCharging);
        
        // Call Tauri backend if available
        if (window.__TAURI__) {
            try {
                await window.__TAURI__.invoke('toggle_charging');
            } catch (error) {
                this.addLogEntry('error', `Failed to toggle charging: ${error}`);
            }
        }
    }

    async setBatteryLevel(level) {
        this.batteryLevel = level;
        this.addLogEntry('info', `Battery level set to ${level}%`);

        // Update UI
        this.updateBatteryDisplay(level, this.isCharging);

        // Call Tauri backend if available
        if (window.__TAURI__) {
            try {
                await window.__TAURI__.invoke('set_battery_level', { level });
            } catch (error) {
                this.addLogEntry('error', `Failed to set battery level: ${error}`);
            }
        }
    }

    async triggerVibration() {
        console.log('🎯 Vibration triggered');

        // Add console logging
        const timestamp = new Date().toLocaleTimeString();
        this.addLogEntry('vibration', `VIBRATION - ${timestamp}`);

        // Show vibration visual effect
        this.showVibrationEffect();

        // Call Tauri backend if available and connected to real ring
        if (window.__TAURI__ && this.isConnectedToRealRing) {
            try {
                await window.__TAURI__.invoke('trigger_vibration');
                this.addLogEntry('info', `Sent vibration to real ring`);
            } catch (error) {
                this.addLogEntry('error', `Failed to trigger vibration: ${error}`);
            }
        } else {
            this.addLogEntry('info', 'Vibration simulated (no real ring connected)');
        }
    }

    showVibrationEffect() {
        const ringVisual = document.querySelector('.ring-visual');
        if (ringVisual) {
            ringVisual.classList.add('vibrating');
            setTimeout(() => {
                ringVisual.classList.remove('vibrating');
            }, 300);
        }
    }

    showBatteryInfo() {
        this.addLogEntry('info', 'Battery info requested');
        
        // Call Tauri backend if available
        if (window.__TAURI__) {
            window.__TAURI__.invoke('get_battery_info').then(info => {
                this.addLogEntry('info', `Battery: ${info.level}%, ${info.voltage}V, ${info.temperature}°C, ${info.health}`);
            }).catch(error => {
                this.addLogEntry('error', `Failed to get battery info: ${error}`);
            });
        }
    }

    updateGestureDisplay(gesture, confidence) {
        this.currentGesture = gesture;
        this.gestureConfidence = confidence;
        
        document.querySelector('.gesture-name').textContent = gesture;
        document.querySelector('.gesture-confidence').textContent = `${confidence}%`;
        
        // Update gesture icon
        const iconMap = {
            'tap': 'fa-hand-pointer',
            'double-tap': 'fa-hand-pointer',
            'hold': 'fa-hand-rock',
            'slide': 'fa-hand-paper',
            'tilt': 'fa-hand-scissors',
            'Ready': 'fa-hand-paper'
        };
        
        const gestureIcon = document.getElementById('gesture-icon');
        if (gestureIcon) {
            gestureIcon.className = `fas ${iconMap[gesture] || 'fa-hand-paper'}`;
        }
    }

    updateBatteryDisplay(level, charging, voltage = null, temperature = null, health = null) {
        this.batteryLevel = level;
        this.isCharging = charging;
        
        // Update battery level bar
        const batteryLevelEl = document.getElementById('battery-level');
        batteryLevelEl.style.width = `${level}%`;
        
        // Update battery percentage
        document.getElementById('battery-percentage').textContent = `${level}%`;
        
        // Update charging icon
        const chargingIcon = document.getElementById('charging-icon');
        if (charging) {
            chargingIcon.classList.add('active');
        } else {
            chargingIcon.classList.remove('active');
        }
        
        // Update additional info if provided
        if (voltage !== null) {
            document.getElementById('battery-voltage').textContent = `${voltage.toFixed(2)}V`;
        }
        if (temperature !== null) {
            document.getElementById('battery-temp').textContent = `${temperature.toFixed(1)}°C`;
        }
        if (health !== null) {
            document.getElementById('battery-health').textContent = health;
        }
        
        // Update battery color based on level
        if (level <= 20) {
            batteryLevelEl.style.background = 'var(--error-color)';
        } else if (level <= 50) {
            batteryLevelEl.style.background = 'var(--warning-color)';
        } else {
            batteryLevelEl.style.background = 'var(--success-color)';
        }
    }

    updateBLEStatus(status, connectedClients = 0) {
        document.getElementById('ble-detail').textContent = status;
        document.getElementById('client-count').textContent = connectedClients;

        const bleStatus = document.getElementById('ble-status');
        const connectionStatus = document.getElementById('connection-status');

        if (connectedClients > 0) {
            this.isConnected = true;
            connectionStatus.innerHTML = '<i class="fas fa-wifi"></i><span>Connected</span>';
            connectionStatus.style.color = 'var(--success-color)';
        } else {
            this.isConnected = false;
            connectionStatus.innerHTML = '<i class="fas fa-wifi"></i><span>Disconnected</span>';
            connectionStatus.style.color = 'var(--text-secondary)';
        }
    }

    animateRing(gesture) {
        const gestureIcon = document.getElementById('gesture-icon');
        gestureIcon.style.animation = 'gesture-activate 0.6s ease-in-out';

        setTimeout(() => {
            gestureIcon.style.animation = '';
        }, 600);
    }

    animateHaptic(pattern, intensity) {
        const dots = document.querySelectorAll('.haptic-dot');
        const duration = pattern === 'pulse' ? 800 : pattern === 'buzz' ? 500 : 300;

        dots.forEach((dot, index) => {
            setTimeout(() => {
                dot.classList.add('active');
                setTimeout(() => {
                    dot.classList.remove('active');
                }, duration / 2);
            }, index * 100);
        });
    }

    displayHapticFeedback(pattern, intensity, duration) {
        this.addLogEntry('info', `Haptic feedback: ${pattern} (${Math.round(intensity * 100)}%, ${duration}ms)`);
        this.animateHaptic(pattern, intensity);
    }

    addLogEntry(type, message) {
        const timestamp = new Date().toLocaleTimeString('en-US', { hour12: false });
        const entry = { timestamp, type, message };

        this.logEntries.push(entry); // Add to end (newest at bottom)
        if (this.logEntries.length > 100) {
            this.logEntries.shift(); // Remove from beginning
        }

        this.updateLogDisplay();
    }

    updateLogDisplay() {
        const logContainer = document.getElementById('activity-log');
        if (!logContainer) {
            console.warn('Activity log container not found, skipping log display update');
            return;
        }

        logContainer.innerHTML = this.logEntries.map(entry => `
            <div class="log-entry">
                <span class="timestamp">${entry.timestamp}</span>
                <span class="level ${entry.type}">${entry.type.toUpperCase()}</span>
                <span class="event">${entry.message}</span>
            </div>
        `).join('');

        // Auto-scroll to the latest log message (bottom, since newest entries are last)
        const lastEntry = logContainer.lastElementChild;
        if (lastEntry) {
            lastEntry.scrollIntoView({
                behavior: 'smooth',
                block: 'end',
                inline: 'nearest'
            });
        }

        // Ensure container scrolls to bottom for newest entries
        setTimeout(() => {
            logContainer.scrollTop = logContainer.scrollHeight;
        }, 50);
    }

    updateUI() {
        this.updateGestureDisplay(this.currentGesture, this.gestureConfidence);
        this.updateBatteryDisplay(this.batteryLevel, this.isCharging);
        this.updateBLEStatus('Advertising', 0);
    }

    startStatusUpdates() {
        // Simulate periodic status updates
        setInterval(() => {
            if (!this.isCharging && this.batteryLevel > 0) {
                // Simulate battery drain
                if (Math.random() < 0.1) { // 10% chance per interval
                    this.batteryLevel = Math.max(0, this.batteryLevel - 1);
                    this.updateBatteryDisplay(this.batteryLevel, this.isCharging);
                }
            } else if (this.isCharging && this.batteryLevel < 100) {
                // Simulate battery charging
                if (Math.random() < 0.3) { // 30% chance per interval
                    this.batteryLevel = Math.min(100, this.batteryLevel + 1);
                    this.updateBatteryDisplay(this.batteryLevel, this.isCharging);

                    if (this.batteryLevel >= 100) {
                        this.isCharging = false;
                        this.addLogEntry('info', 'Battery fully charged');
                        this.updateBatteryDisplay(this.batteryLevel, this.isCharging);
                    }
                }
            }
        }, 2000); // Update every 2 seconds

        // Simulate random gesture confidence fluctuations when ready
        setInterval(() => {
            if (this.currentGesture === 'Ready') {
                const confidence = 95 + Math.floor(Math.random() * 5);
                this.updateGestureDisplay('Ready', confidence);
            }
        }, 5000);
    }
}

// Initialize the application when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    window.ringSimulator = new RingSimulator();

    // Add some initial log entries
    setTimeout(() => {
        window.ringSimulator.addLogEntry('success', 'BLE peripheral started');
        window.ringSimulator.addLogEntry('info', 'Advertising as "Haptic Harmony Ring Simulator"');
        window.ringSimulator.addLogEntry('info', 'Ready for gestura.app connection');
    }, 1000);
});

// Initialize app when DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    console.log('🚀 DOM loaded, initializing app...');

    // Create and initialize app
    const app = new RingSimulator();
    app.init();

    // Make app globally accessible for debugging
    window.app = app;

    // Export for Tauri integration
    if (window.__TAURI__) {
        window.ringSimulator = app;
    }

    // Add direct event listeners as backup
    console.log('🔧 Setting up direct event listeners...');

    // Interface switching
    const ringTab = document.getElementById('ring-tab');
    const configTab = document.getElementById('config-tab');

    if (ringTab) {
        ringTab.addEventListener('click', () => {
            console.log('🎯 Ring tab clicked directly');
            app.switchInterface('ring');
        });
    }

    if (configTab) {
        configTab.addEventListener('click', () => {
            console.log('🎯 Config tab clicked directly');
            app.switchInterface('config');
        });
    }

    // Console toggle
    const consoleToggle = document.getElementById('console-toggle');
    if (consoleToggle) {
        consoleToggle.addEventListener('click', () => {
            console.log('🎯 Console toggle clicked directly');
            app.toggleConsole();
        });
    }

    // Tilt toggle
    const tiltToggle = document.getElementById('tilt-toggle');
    if (tiltToggle) {
        tiltToggle.addEventListener('click', () => {
            console.log('🎯 Tilt toggle clicked directly');
            app.toggleTiltMode();
        });
    }

    console.log('✅ App initialization complete');
});
