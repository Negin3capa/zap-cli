const fs = require('fs');

class ImageViewer {
    constructor() {
        this.isKitty = process.env.TERM && process.env.TERM.includes('kitty');
    }

    /**
     * Converts an image buffer to Kitty graphics protocol escape codes.
     * @param {Buffer} buffer - The image buffer.
     * @param {Object} options - Options for display (e.g., id, placement).
     * @returns {string} - The escape sequence string.
     */
    getKittyImageString(buffer, options = {}) {
        if (!buffer) return '';
        
        const base64 = buffer.toString('base64');
        const chunks = [];
        const chunkSize = 4096;
        
        for (let i = 0; i < base64.length; i += chunkSize) {
            chunks.push(base64.substring(i, i + chunkSize));
        }

        let result = '';
        const id = options.id || Math.floor(Math.random() * 1000000);
        
        chunks.forEach((chunk, index) => {
            const isLast = index === chunks.length - 1;
            const header = [];
            
            if (index === 0) {
                header.push('a=T'); // Transmit and display
                header.push('f=100'); // PNG format (100) - Kitty auto-detects usually but good to specify if known. 
                // However, without mime type detection, 100 might be wrong for JPG. 
                // Kitty docs say: f=100 is PNG, f=32 is JPG. 
                // We'll try to auto-detect or omit 'f' to let Kitty detect magic numbers if supported, 
                // but usually 'f' is needed or 't=d' (transfer direct).
                // Actually, Kitty supports autodetection if 'f' is omitted and 't=d' is used?
                // Let's use standard transmission 't=d' (default) and 'f=100' is generic payload? 
                // No, f=100 is specifically PNG. 
                // Let's assume we pass the format in options or try to detect.
                
                if (options.format === 'png') header.push('f=100');
                else if (options.format === 'jpg' || options.format === 'jpeg') header.push('f=32');
                
                header.push('t=d');
                header.push(`i=${id}`);
                // Placement options could be added here
            }
            
            header.push(`m=${isLast ? 0 : 1}`); // More chunks?
            
            // Payload
            result += `\x1b_G${header.join(',')};${chunk}\x1b\\`;
        });

        return result;
    }

    display(buffer, format = 'png') {
        if (this.isKitty) {
            process.stdout.write(this.getKittyImageString(buffer, { format }));
        } else {
            console.log('[Image display not supported in this terminal]');
        }
    }
}

module.exports = new ImageViewer();
