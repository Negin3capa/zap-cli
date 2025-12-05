# Quick Fix for "Port 8080 in use" Error

If you see this error:

```
❌ Error: Port 8080 is already in use by another application
```

## Solution: Kill the process using port 8080

```bash
sudo lsof -ti:8080 | xargs kill -9
```

Then run `./zaptui` again.

## What happened?

Another program on your system is using port 8080 (the port ZapTUI's WhatsApp service needs).

## Alternative: Use a different port

1. **Edit whatsapp-service/server.js**:
   - Change line 83 from `port: 8080` to `port: 8081`

2. **Edit config.toml** (or create it from config.example.toml):

   ```toml
   [whatsapp]
   service_url = "ws://localhost:8081"  # Changed from 8080 to 8081
   ```

3. Run `./zaptui` again
