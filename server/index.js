import express from 'express';
const app = express();

import https from 'https';
import http from 'http';

const server = (process.env.NODE_ENV === 'production') ?
  https.createServer({
    key: fs.readFileSync("/etc/letsencrypt/live/zacknoyes.au/privkey.pem"),
    cert: fs.readFileSync("/etc/letsencrypt/live/zacknoyes.au/fullchain.pem")
  }, app)
  : http.createServer(app);

if (process.env.NODE_ENV === 'production') {
  server.listen(443, () => {
    console.log('listening on *:443');
  });
  const redirectApp = express();
  const redirectServer = http.createServer(redirectApp);
  redirectApp.use(function(req, res, next) {
    if (process.env.NODE_ENV == 'production' && !req.secure) {
      return res.redirect('https://' + req.headers.host + req.url);
    }
    next();
  });
  redirectServer.listen(80, () => {
    console.log('listening on *:80');
  });
} else {
  server.listen(8080, () => {
    console.log('listening on *:8080');
  });
}

import fs from 'fs';

import { Server } from "socket.io";
const io = new Server(server);

import { createClient } from 'redis';
const redisClient = createClient({
  url: 'redis://127.0.0.1:6380'
});
redisClient.on('error', err => console.log('Redis Client Error', err));
await redisClient.connect();

import path from 'path';
import { fileURLToPath } from 'url';
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

import crypto from 'crypto';

import 'log-timestamp';

const CHANCE_OF_BONUS = 0.25;

app.use(express.static(path.join(__dirname,'dist')));
app.use(express.json());

app.get('/api/newUser', async (req, res) => {
  console.log("NEW USER");

  // DoS protection. Only allow 10 users per IP address.
  // Disabled for now because it's not working.
  // let ip = req.socket.remoteAddress;
  // let numUsers = await redisClient.sCard('users:ip:' + ip);
  // if (numUsers >= 10) {
  //   res.sendStatus(429);
  //   return;
  // }

  let id = await redisClient.incr('next_user_id');
  console.log("NEW USER ID " + id);

  let password = crypto.randomBytes(64).toString('hex');
  await redisClient.hSet('user:' + id, 'password', password);
  res.status(200).send({
    id: id,
    password: password
  });
});

app.post('/api/result', async (req, res) => {
  console.log("RESULT");
  if (
    req.body == undefined
    || req.body.id == undefined
    || req.body.password == undefined
    || req.body.date == undefined
    || req.body.numMoves == undefined
    || req.body.winner == undefined
  ) {
    res.sendStatus(400);
    return;
  }
  let password = await redisClient.hGet('user:' + req.body.id, 'password');
  if (password != req.body.password) {
    res.sendStatus(401);
    return;
  }
  let check = await redisClient.sAdd('users_played:date:' + req.body.date, req.body.id);
  if (check == 0) {
    res.sendStatus(400);
    return;
  }
  console.log("RESULT " + req.body.id + ", " + req.body.date + ", " + req.body.numMoves + ", " + req.body.winner);
  await redisClient.rPush('raw_results', JSON.stringify(req.body));
  let bucket = Math.min(Math.floor(req.body.numMoves / 10), 10);
  if (req.body.winner == "white") {
    await redisClient.hIncrBy('user:' + req.body.id, 'num_wins', 1);
    await redisClient.hIncrBy('date:' + req.body.date, 'num_wins', 1);
    await redisClient.hIncrBy('user:' + req.body.id, 'win_moves_bucket:' + bucket, 1);
    await redisClient.hIncrBy('date:' + req.body.date, 'win_moves_bucket:' + bucket, 1);
  } else if (req.body.winner == "black") {
    await redisClient.hIncrBy('user:' + req.body.id, 'num_losses', 1);
    await redisClient.hIncrBy('date:' + req.body.date, 'num_losses', 1);
    await redisClient.hIncrBy('user:' + req.body.id, 'loss_moves_bucket:' + bucket, 1);
    await redisClient.hIncrBy('date:' + req.body.date, 'loss_moves_bucket:' + bucket, 1);
  } else {
    await redisClient.hIncrBy('user:' + req.body.id, 'num_draws', 1);
    await redisClient.hIncrBy('date:' + req.body.date, 'num_draws', 1);
  }
  res.sendStatus(200);
});

app.get('/api/stats/:id/:date', async (req, res) => {
  console.log("STATS " + req.params.id + ", " + req.params.date);
  let uStats = await redisClient.hGetAll('user:' + req.params.id);
  if (uStats == null) {
    res.sendStatus(404);
    return;
  }
  let dStats = await redisClient.hGetAll('date:' + req.params.date);
  if (dStats == null) {
    res.sendStatus(404);
    return;
  }
  var uWinMoves = [];
  var uLossMoves = [];
  var dWinMoves = [];
  var dLossMoves = [];
  for (let i = 0; i <= 10; i++) {
    uWinMoves.push(uStats['win_moves_bucket:' + i]);
    uLossMoves.push(uStats['loss_moves_bucket:' + i]);
    dWinMoves.push(dStats['win_moves_bucket:' + i]);
    dLossMoves.push(dStats['loss_moves_bucket:' + i]);
  }
  res.status(200).send({
    uNumWins: uStats.num_wins,
    uNumLosses: uStats.num_losses,
    uNumDraws: uStats.num_draws,
    dNumWins: dStats.num_wins,
    dNumLosses: dStats.num_losses,
    dNumDraws: dStats.num_draws,
    uWinMoves: uWinMoves,
    uLossMoves: uLossMoves,
    dWinMoves: dWinMoves,
    dLossMoves: dLossMoves
  });
});

// Stuff for the online games

var games = {};

io.on('connection', (socket) => {
  console.log("CONNECTION " + socket.id);
  
  socket.on('host', () => {
    console.log("HOST " + socket.id);
    var tries = 0;
    while (true) {
      var code = Math.floor(Math.random() * 900 + 100);
      if (games[code] == undefined) {
        games[code] = { host: socket.id, guest: undefined };
        console.log("HOSTED " + socket.id + " " + code);
        socket.emit('hosted', code);
        break;
      }
      tries++;
      if (tries > 10) {
        console.log("HOST FAILED " + socket.id);
        socket.emit('hostFailed');
        break;
      }
    }
  });

  socket.on('join', (code) => {
    var hostStarts = Math.random() < 0.5;
    console.log("JOIN " + socket.id + " " + code);
    if (games[code] == undefined || games[code].guest != undefined) {
      socket.emit('joinFailed');
    } else {
      games[code].guest = socket.id;
      socket.emit('joined', !hostStarts);
      io.to(games[code].host).emit('opponentJoined', hostStarts);
    }
  });

  socket.on('move', (fromX, fromY, toX, toY, p) => {
    console.log("MOVE " + socket.id + " " + fromX + " " + fromY + " " + toX + " " + toY + " " + p);
    var isBonus = Math.random() < CHANCE_OF_BONUS;
    for (var code in games) {
      if (games[code].host == socket.id) {
        io.to(games[code].guest).emit('opponentMove', fromX, fromY, toX, toY, p);
        io.to(games[code].guest).emit('isBonus', isBonus);
        io.to(games[code].host).emit('isBonus', isBonus);
      }
      if (games[code].guest == socket.id) {
        io.to(games[code].host).emit('opponentMove', fromX, fromY, toX, toY, p);
        io.to(games[code].guest).emit('isBonus', isBonus);
        io.to(games[code].host).emit('isBonus', isBonus);
      }
    }
  });

  socket.on('disconnect', () => {
    console.log("DISCONNECT " + socket.id);
    for (var code in games) {
      if (games[code].host == socket.id || games[code].guest == socket.id) {
        if (games[code].host != undefined) {
          io.to(games[code].host).emit('opponentDisconnected');
        }
        if (games[code].guest != undefined) {
          io.to(games[code].guest).emit('opponentDisconnected');
        }
        delete games[code];
      }
    }
  });

});
