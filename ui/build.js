#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Simple build script to copy files to dist
const srcDir = path.join(__dirname, 'src');
const distDir = path.join(__dirname, 'dist');

// Create dist directory if it doesn't exist
if (!fs.existsSync(distDir)) {
    fs.mkdirSync(distDir, { recursive: true });
}

// Copy files
const files = ['index.html', 'styles.css', 'app.js'];

files.forEach(file => {
    const srcPath = path.join(srcDir, file);
    const distPath = path.join(distDir, file);
    
    if (fs.existsSync(srcPath)) {
        fs.copyFileSync(srcPath, distPath);
        console.log(`Copied ${file} to dist/`);
    } else {
        console.warn(`Warning: ${file} not found in src/`);
    }
});

console.log('Build complete!');
