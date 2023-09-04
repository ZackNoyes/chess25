
# This only works with the configuration that I've set up both on my machine
# and the remote.

echo "Delete the local dist directory..."
rm -r www/dist

echo "Compiling everything locally..."
# I'm pretty sure the line below is a security issue
export NODE_OPTIONS=--openssl-legacy-provider
cd server && npm run build && cd ..

echo "Killing redis-server and node on remote..."
ssh do-droplet pkill redis-server
ssh do-droplet pkill redis-server
ssh do-droplet pkill node

echo "Deleting old files on remote..."
ssh do-droplet rm -r server

echo "Deleting node_modules on this machine..."
rm -r server/node_modules

echo "Copying files to remote..."
scp -r server do-droplet:~/server

echo "Copying sym-linked dist directory to remote..."
ssh do-droplet mkdir server/dist
scp -r server/dist/* do-droplet:~/server/dist

echo "Installing dependencies on remote..."
ssh do-droplet "cd server && npm install"

echo "Done! Now follow these steps to start the server:"
echo "1. ssh do-droplet"
echo "2. cd data"
echo "3. redis-server --port 6380 >> redis.log 2>&1 &"
echo "4. cd .."
echo "5. export NODE_ENV=production"
echo "6. cd server"
echo "7. node index.js >> ../node.log 2>&1 &"
echo "8. exit"
echo "9. Try connecting at: https://chess25.zacknoyes.au"