import { JSInterface } from "random-chess";

// // Construct the universe, and get its width and height.
// const universe = Universe.new();
// const width = universe.width();
// const height = universe.height();

// Colour constants
const BLACK_SQUARE_COLOR = 'rgb(176, 124, 92)';
const WHITE_SQUARE_COLOR = 'rgb(240, 211, 175)';
const SELECTED_COLOR = 'rgba(20, 126, 72, 0.7)';
const HOVER_COLOR = 'rgba(150, 150, 150, 0.4)';
const ACTIVE_COLOR = 'rgba(154, 195, 69, 0.4)';

const CHANCE_OF_BONUS = 0.25;

// Give the canvas room for all of our cells and a 1px border
// around each of them.
const canvas = document.getElementById("game-canvas");
const CANVAS_SIZE = 600;
const SQUARE_SIZE = CANVAS_SIZE / 8;
canvas.width = CANVAS_SIZE;
canvas.height = CANVAS_SIZE;
var ctx = canvas.getContext('2d');
const socket = io({autoConnect: false});

var wasmInterface = undefined;
var gameType = undefined; // "local", "networkH", "networkG", "ai", "aiai", "daily"
var awaitingMoveFrom = undefined; // "ui", "network", "ai"

// Daily game things
var dateString = undefined;
var rng = undefined;
var turns = [];
window.turns = turns;

var hoveredSquare = undefined;
var selectedPiece = undefined;
var activeSquares = [];

function flip(rank) { return gameType != "networkG" ? 7 - rank : rank; }

function draw() {

  checkInteractive();

  if (wasmInterface == undefined) return;

  drawBoard();
  drawSelected();
  drawActiveSquares();
  drawPieces();
  drawMoves();
  drawHover();
  updateStatus();

};

// Function to draw the chess board
function drawBoard() {
  for (let file = 0; file < 8; file++) {
    for (let rank = 0; rank < 8; rank++) {
      if ((file + flip(rank)) % 2 == 0) {
        ctx.fillStyle = BLACK_SQUARE_COLOR;
      } else {
        ctx.fillStyle = WHITE_SQUARE_COLOR;
      }
      ctx.fillRect(file * SQUARE_SIZE, rank * SQUARE_SIZE, SQUARE_SIZE, SQUARE_SIZE);
    }
  }
}

function drawSelected() {
  if (selectedPiece != undefined) {
    ctx.fillStyle = SELECTED_COLOR;
    ctx.fillRect(selectedPiece.x * SQUARE_SIZE, selectedPiece.y * SQUARE_SIZE, SQUARE_SIZE, SQUARE_SIZE);
  }
}

function drawPieces() {

  for (let file = 0; file < 8; file++) {
    for (let rank = 0; rank < 8; rank++) {
      var piece = wasmInterface.js_piece(file, flip(rank));
      if (piece != undefined) {
        let img = images[urlForPiece(piece)];
        ctx.drawImage(img, file * SQUARE_SIZE, rank * SQUARE_SIZE, SQUARE_SIZE, SQUARE_SIZE);
      }
    }
  }
}

function drawMoves() {
  if (selectedPiece != undefined) {
    for (let move of wasmInterface.js_moves_from(selectedPiece.x, flip(selectedPiece.y))) {
      ctx.beginPath();
      ctx.fillStyle = SELECTED_COLOR;
      ctx.arc(
        move[2] * SQUARE_SIZE + SQUARE_SIZE / 2,
        flip(move[3]) * SQUARE_SIZE + SQUARE_SIZE / 2,
        SQUARE_SIZE / 6, 0, 2 * Math.PI
      );
      ctx.fill();
    }
  }
}

function drawHover() {
  if (hoveredSquare != undefined) {
    ctx.fillStyle = HOVER_COLOR;
    ctx.fillRect(hoveredSquare.x * SQUARE_SIZE, hoveredSquare.y * SQUARE_SIZE, SQUARE_SIZE, SQUARE_SIZE);
  }
}

function drawActiveSquares() {
  for (let square of activeSquares) {
    ctx.fillStyle = ACTIVE_COLOR;
    ctx.fillRect(square.x * SQUARE_SIZE, flip(square.y) * SQUARE_SIZE, SQUARE_SIZE, SQUARE_SIZE);
  }
}

function drawHistory(context, i) {
  for (let file = 0; file < 8; file++) {
    for (let rank = 0; rank < 8; rank++) {
      if ((file + flip(rank)) % 2 == 0) {
        context.fillStyle = BLACK_SQUARE_COLOR;
      } else {
        context.fillStyle = WHITE_SQUARE_COLOR;
      }
      context.fillRect(file * SQUARE_SIZE, rank * SQUARE_SIZE, SQUARE_SIZE, SQUARE_SIZE);
      if (wasmInterface.js_history_was_hot(file, flip(rank), i)) {
        context.fillStyle = ACTIVE_COLOR;
        context.fillRect(file * SQUARE_SIZE, rank * SQUARE_SIZE, SQUARE_SIZE, SQUARE_SIZE);
      }
      var piece = wasmInterface.js_history_piece(file, flip(rank), i);
      if (piece != undefined) {
        let img = images[urlForPiece(piece)];
        context.drawImage(img, file * SQUARE_SIZE, rank * SQUARE_SIZE, SQUARE_SIZE, SQUARE_SIZE);
      }
    }
  }
}

function updateStatus() {

  let statusLabel = document.getElementById("status");
  statusLabel.innerHTML = "&nbsp;";

  switch (wasmInterface.js_status()) {
    case "in progress": {
      if (awaitingMoveFrom == undefined) {
        statusLabel.innerHTML = "<span id='dice' style='display: inline-block;'>🎲</span>";
        break;
      }
      switch (wasmInterface.js_get_side_to_move()) {
        case "white": {
          switch (gameType) {
            case "local": case "aiai": statusLabel.innerHTML = "white's move"; break;
            case "networkH": statusLabel.innerHTML = "your move"; break;
            case "networkG": statusLabel.innerHTML = "waiting for opponent"; break;
            case "ai": case "daily": statusLabel.innerHTML = "your move"; break;
          }
          break;
        }
        case "black": {
          switch (gameType) {
            case "local": case "aiai": statusLabel.innerHTML = "black's move"; break;
            case "networkH": statusLabel.innerHTML = "waiting for opponent"; break;
            case "networkG": statusLabel.innerHTML = "your move"; break;
            case "ai": case "daily": statusLabel.innerHTML = "waiting for computer"; break;
          }
          break;
        }
      }
      break;
    }
    case "white": {
      switch (gameType) {
        case "local": case "aiai": statusLabel.innerHTML = "white wins"; break;
        case "networkH": statusLabel.innerHTML = "you win"; break;
        case "networkG": statusLabel.innerHTML = "opponent wins"; break;
        case "ai": case "daily": statusLabel.innerHTML = "you win"; break;
      }
      break;
    }
    case "black": {
      switch (gameType) {
        case "local": case "aiai": statusLabel.innerHTML = "black wins"; break;
        case "networkH": statusLabel.innerHTML = "opponent wins"; break;
        case "networkG": statusLabel.innerHTML = "you win"; break;
        case "ai": case "daily": statusLabel.innerHTML = "computer wins"; break;
      }
      break;
    }
    case "draw": statusLabel.innerHTML = "draw"; break;
  }
  
};

const urlForPiece = (piece) => {
  var t = piece.toUpperCase();
  var c = t == piece ? 'w' : 'b';
  return 'images/cburnett/' + c + t + '.svg';
}
const images = {};
const loadImage = piece =>
  new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = reject;
    img.src = urlForPiece(piece);
    images[urlForPiece(piece)] = img;
  });
const PIECES = [
  'P', 'N', 'B', 'R', 'Q', 'K',
  'p', 'n', 'b', 'r', 'q', 'k'
];
Promise.all(PIECES.map(loadImage)).then(pieces => {
  draw();
});

function checkInteractive() {
  if (awaitingMoveFrom != "ui") {
    hoveredSquare = undefined;
    selectedPiece = undefined;
    return false;
  }
  return true;
}

function makeAlert(alertType, alertText, timeout) {
  const alert = document.getElementById("alert").cloneNode(true);
  alert.style.display = "block";
  alert.classList.add("alert-" + alertType);
  alert.innerHTML = alertText;
  document.getElementById("alerts").appendChild(alert);
  setTimeout(() => {
    new bootstrap.Alert(alert).close();
  }, timeout);
}

canvas.addEventListener('pointermove', event => {

  if (!checkInteractive()) { return; }

  let rect = canvas.getBoundingClientRect();
  let x = Math.floor(event.offsetX * canvas.width / rect.width / SQUARE_SIZE);
  let y = Math.floor(event.offsetY * canvas.height / rect.height / SQUARE_SIZE);

  hoveredSquare = { x: x, y: y };

  draw();
  
});

canvas.addEventListener('pointerout', event => {

  if (!checkInteractive()) { return; }

  hoveredSquare = undefined;
  draw();
});

canvas.addEventListener('pointerup', event => {

  if (!checkInteractive()) { return; }

  if (wasmInterface.js_status() != "in progress") { return; }

  // Get the x and y coordinates of the canvas event
  let rect = canvas.getBoundingClientRect();
  let x = Math.floor(event.offsetX * canvas.width / rect.width / SQUARE_SIZE);
  let y = Math.floor(event.offsetY * canvas.height / rect.height / SQUARE_SIZE);
  
  if (selectedPiece == undefined) {
    if (wasmInterface.js_piece_color(x, flip(y)) == wasmInterface.js_get_side_to_move()) {
      selectedPiece = { x: x, y: y };
    }
  } else if (selectedPiece.x == x && selectedPiece.y == y) {
    selectedPiece = undefined;
  } else if (wasmInterface.js_check_move(selectedPiece.x, flip(selectedPiece.y), x, flip(y)) != undefined) {
    if (wasmInterface.js_check_move(selectedPiece.x, flip(selectedPiece.y), x, flip(y))) {
      const myModal = bootstrap.Modal.getOrCreateInstance(document.getElementById('promotionSelect'));
      for (let piece of [1, 2, 3, 4]) {
        document.getElementById("promoOption" + piece).firstChild.src =
          urlForPiece(
            wasmInterface.js_get_side_to_move() == "white" ?
            PIECES[piece] : PIECES[piece + 6]
          );
        document.getElementById("promoOption" + piece).onclick = () => {
          registerMove(selectedPiece.x, flip(selectedPiece.y), x, flip(y), piece);
          selectedPiece = undefined;
          draw();
        };
      }
      myModal.show();
    } else {
      registerMove(selectedPiece.x, flip(selectedPiece.y), x, flip(y), undefined);
      selectedPiece = undefined;
    }
  } else {
    if (wasmInterface.js_piece_color(x, flip(y)) == wasmInterface.js_get_side_to_move()) {
      selectedPiece = { x: x, y: y };
    } else {
      selectedPiece = undefined;
    }
  }

  draw();

});

function switchToGameUI() {
  document.getElementById("startOptions").style.display = "none";
  document.getElementById("game").style.display = "block";
}

function initLocalGame() {
  switchToGameUI();

  let whiteStarts = Math.random() < 0.5;
  if (whiteStarts) { makeAlert("info", "White gets to move first", 5000); }
  else { makeAlert("info", "Black gets to move first", 5000); }

  wasmInterface = JSInterface.js_initial_interface(whiteStarts);
  gameType = "local";
  awaitingMoveFrom = "ui";
  draw();
}
window.initLocalGame = initLocalGame;

function initiateHosting() {
  socket.connect();
  socket.emit("host");
  document.getElementById("hostCode").innerHTML = "loading";
  document.getElementById("hostModal").addEventListener("hidden.bs.modal", () => {
    if (gameType == undefined) socket.disconnect();
  });
}
window.initiateHosting = initiateHosting;

socket.on("hosted", (id) => {
  document.getElementById("hostCode").innerHTML = id;
});

socket.on("opponentJoined", (iStart) => {
  switchToGameUI();
  wasmInterface = JSInterface.js_initial_interface(iStart);
  gameType = "networkH";
  if (iStart) {
    makeAlert("success", "You get to move first", 5000);
    awaitingMoveFrom = "ui";
  } else {
    makeAlert("danger", "Opponent gets to move first", 5000);
    awaitingMoveFrom = "network";
  }
  bootstrap.Modal.getOrCreateInstance(document.getElementById("hostModal")).hide();
  draw();
});

function joinGame(code) {
  socket.connect();
  socket.emit("join", code);
  document.getElementById("joinModal").addEventListener("hidden.bs.modal", () => {
    if (gameType == undefined) socket.disconnect();
  });
}
window.joinGame = joinGame;

socket.on("joined", (iStart) => {
  switchToGameUI();
  wasmInterface = JSInterface.js_initial_interface(!iStart);
  gameType = "networkG";
  if (iStart) {
    makeAlert("success", "You get to move first", 5000);
    awaitingMoveFrom = "ui";
  } else {
    makeAlert("danger", "Opponent gets to move first", 5000);
    awaitingMoveFrom = "network";
  }
  bootstrap.Modal.getOrCreateInstance(document.getElementById("joinModal")).hide();
  draw();
});

socket.on("joinFailed", () => {
  bootstrap.Modal.getOrCreateInstance(document.getElementById("joinModal")).hide();
  bootstrap.Modal.getOrCreateInstance(document.getElementById("joinFailedModal")).show();
});

socket.on("opponentDisconnected", () => {
  bootstrap.Modal.getOrCreateInstance(document.getElementById("opponentDisconnectedModal")).show();
  awaitingMoveFrom = undefined;
  socket.disconnect();
  draw();
});

socket.on("opponentMove", (fromX, fromY, toX, toY, p) => {
  registerMove(fromX, fromY, toX, toY, p);
});

socket.on("isBonus", (isBonus) => {
  setTimeout(() => {
    registerRand(isBonus);
  }, 1000);
});

function initAIGame() {
  switchToGameUI();

  let whiteStarts = Math.random() < 0.5;
  if (whiteStarts) { makeAlert("info", "You get to move first", 5000); }
  else { makeAlert("info", "The computer gets to move first", 5000); }

  wasmInterface = JSInterface.js_initial_interface(whiteStarts);
  gameType = "ai";
  awaitingMoveFrom = whiteStarts ? "ui" : "ai";
  draw();

  if (awaitingMoveFrom == "ai") { dispatchAIMove(); }
}
window.initAIGame = initAIGame;

function initAIvsAIGame() {
  switchToGameUI();

  let whiteStarts = Math.random() < 0.5;
  if (whiteStarts) { makeAlert("info", "White gets to move first", 5000); }
  else { makeAlert("info", "Black gets to move first", 5000); }

  wasmInterface = JSInterface.js_initial_interface(whiteStarts);
  gameType = "aiai";
  awaitingMoveFrom = "ai"
  draw();

  dispatchAIMove();
}
window.initAIvsAIGame = initAIvsAIGame;

function initDailyGame() {

  let date = new Date();
  dateString = date.getFullYear() + "_" + date.getMonth() + "_" + date.getDate();
  rng = new Math.seedrandom(dateString);

  if (localStorage.getItem("mostRecentDailyGame") == dateString) {
    bootstrap.Modal.getOrCreateInstance(document.getElementById("dailyStats")).show();
    document.getElementById("dailyModalTitle").innerHTML = "Today's Game";
    document.getElementById("dailyResultText").innerHTML = localStorage.getItem("mostRecentDailyGameResult");
    let updateCounter = setInterval(function() {
      let nextMidnight = new Date();
      nextMidnight.setHours(24,0,0,0);
      let now = new Date();
      let remainingTimeInSeconds = (nextMidnight.getTime() - now.getTime())/1000;
      let hours = Math.floor(remainingTimeInSeconds / 3600);
      var rest = remainingTimeInSeconds - hours * 3600;
      let minutes = Math.floor(rest / 60);
      var rest = rest - minutes * 60;
      let seconds = Math.floor(rest);
      if (remainingTimeInSeconds > 86370) {
        window.location.reload();
      }
      if (document.getElementById("moveHistory") == undefined) {
        clearInterval(interval);
        return;
      }
      document.getElementById("moveHistory").innerHTML = "Tomorrow's game will be available in <b>" + String(hours).padStart(2, '0') + ":" + String(minutes).padStart(2, '0') + ":" + String(seconds).padStart(2, '0') + "</b>";
    }, 100);
    loadStats();
    return;
  }

  switchToGameUI();
  document.getElementById("quitButton").style.display = "none";

  let whiteStarts = rng() < 0.5;
  if (whiteStarts) {
    turns.push("w");;
    makeAlert("info", "You get to move first", 5000);
  } else {
    turns.push("b");
    makeAlert("info", "The computer gets to move first", 5000);
  }

  wasmInterface = JSInterface.js_initial_interface(whiteStarts);
  gameType = "daily";
  awaitingMoveFrom = whiteStarts ? "ui" : "ai";
  draw();

  if (awaitingMoveFrom == "ai") { dispatchAIMove(); }
}
window.initDailyGame = initDailyGame;

function dispatchAIMove() {
  setTimeout(() => { // TODO: properly run this in the background
    let move = wasmInterface.js_get_engine_move();
    registerMove(move[0], move[1], move[2], move[3], move[4]);
  }, 1000);
}

function registerMove(xf, yf, xt, yt, p) {
  console.assert(awaitingMoveFrom != undefined);
  if (awaitingMoveFrom == "ui" && gameType.startsWith("network")) {
    socket.emit("move", xf, yf, xt, yt, p);
  }
  awaitingMoveFrom = undefined;
  wasmInterface.js_apply_move(xf, yf, xt, yt, p);
  activeSquares = [];
  activeSquares.push({ x: xf, y: yf });
  activeSquares.push({ x: xt, y: yt });
  startRandGen();
  draw();
}

var animation = undefined;
function startRandGen() {
  var angle = 0;
  animation = setInterval(() => {
    angle += 3;
    $("#dice").css('transform','rotate(' + angle + 'deg)');
  }, 5);
  if (!gameType.startsWith("network")) {
    setTimeout(() => {
      if (gameType == "daily") {
        registerRand(rng() < CHANCE_OF_BONUS);
        turns.push(wasmInterface.js_get_side_to_move() == "white" ? "w" : "b");
      } else {
        registerRand(Math.random() < CHANCE_OF_BONUS);
      }
    }, 1500);
  }
}

function stopRandGen() {
  clearInterval(animation);
  animation = undefined;
}

function registerRand(isBonus) {

  stopRandGen();

  wasmInterface.js_apply_bonus(isBonus);

  if (isBonus && wasmInterface.js_status() == "in progress") {

    var alertType = "";
    switch (gameType) {
      case "local": case "aiai": alertType = "info"; break;
      case "networkG": {
        if (wasmInterface.js_get_side_to_move() == "white") alertType = "danger";
        else alertType = "success";
        break;
      }
      case "networkH":
      case "ai": case "daily": {
        if (wasmInterface.js_get_side_to_move() == "white") alertType = "success";
        else alertType = "danger";
        break;
      }
    }

    var alertText = "";
    switch (gameType) {
      case "local": case "aiai": alertText = "Bonus turn for " + wasmInterface.js_get_side_to_move(); break;
      case "networkG": {
        if (wasmInterface.js_get_side_to_move() == "white") alertText = "Your opponent got a bonus turn";
        else alertText = "You got a bonus turn";
        break;
      }
      case "networkH": {
        if (wasmInterface.js_get_side_to_move() == "white") alertText = "You got a bonus turn";
        else alertText = "Your opponent got a bonus turn";
        break;
      }
      case "ai": case "daily": {
        if (wasmInterface.js_get_side_to_move() == "white") alertText = "You got a bonus turn";
        else alertText = "The computer got a bonus turn";
        break;
      }
    }

    makeAlert(alertType, alertText, 2000);

  }

  if (wasmInterface.js_status() == "in progress") {
    switch (gameType) {
      case "local": awaitingMoveFrom = "ui"; break;
      case "networkH": awaitingMoveFrom = wasmInterface.js_get_side_to_move() == "white" ? "ui" : "network"; break;
      case "networkG": awaitingMoveFrom = wasmInterface.js_get_side_to_move() == "white" ? "network" : "ui"; break;
      case "ai": case "daily": awaitingMoveFrom = wasmInterface.js_get_side_to_move() == "white" ? "ui" : "ai"; break;
      case "aiai": awaitingMoveFrom = "ai"; break;
    }
  } else {

    var alertType = "";
    switch (gameType) {
      case "local": case "aiai": alertType = "info"; break;
      case "networkG": {
        if (wasmInterface.js_status() == "white") alertType = "danger";
        else if (wasmInterface.js_status() == "black") alertType = "success";
        else alertType = "info";
        break;
      }
      case "networkH":
      case "ai": case "daily": {
        if (wasmInterface.js_status() == "white") alertType = "success";
        else if (wasmInterface.js_status() == "black") alertType = "danger";
        else alertType = "info";
      }
    }

    var alertText = "";
    switch (gameType) {
      case "local": case "aiai": {
        if (wasmInterface.js_status() == "white") alertText = "White won";
        else if (wasmInterface.js_status() == "black") alertText = "Black won";
        else alertText = "Draw";
        break;
      }
      case "networkG": {
        if (wasmInterface.js_status() == "white") alertText = "You lost";
        else if (wasmInterface.js_status() == "black") alertText = "You won";
        else alertText = "Draw";
        break;
      }
      case "networkH":
      case "ai": case "daily": {
        if (wasmInterface.js_status() == "white") alertText = "You won";
        else if (wasmInterface.js_status() == "black") alertText = "You lost";
        else alertText = "Draw";
        break;
      }
    }

    makeAlert(alertType, alertText, 2000);

    socket.disconnect();

    if (gameType == "daily") {

      document.getElementById("quitButton").style.display = "block";

      var request = new XMLHttpRequest();
      request.open("POST", "/api/result", false);
      request.setRequestHeader("Content-Type", "application/json");
      request.send(JSON.stringify({
        "id": getUserCreds()[0],
        "password": getUserCreds()[1],
        "date": dateString,
        "numMoves": turns.length,
        "winner": wasmInterface.js_status()
      }));

      if (request.status != 200) {
        console.log("Error submitting daily game result");
        return;
      }

      localStorage.setItem("mostRecentDailyGame", dateString);

      bootstrap.Modal.getOrCreateInstance(document.getElementById("dailyStats")).show();
      document.getElementById("dailyModalTitle").innerHTML = "Game Over";
      let winner = wasmInterface.js_status() == "white" ? "You" : "The computer";
      let numMoves = turns.length;
      let text = wasmInterface.js_status() == "draw" ? "The game was a draw after " + numMoves + " moves" : winner + " won in " + numMoves + " moves";
      localStorage.setItem("mostRecentDailyGameResult", text);
      localStorage.setItem("mostRecentWinner", winner);
      localStorage.setItem("mostRecentNumMoves", numMoves);
      document.getElementById("dailyResultText").innerHTML = text;
      document.getElementById("moveHistory").innerHTML = "";
      for (let i = 0; i < turns.length; i++) {
        document.getElementById("moveHistory").innerHTML += "<span data-bs-toggle='tooltip' data-bs-title='<canvas class=\"tooltip-canvas\" id=\"tooltipCanvas"+i+"\"></canvas>' class='" + (turns[i] == "w" ? "white" : "black") + "-box'></span>";
      }
      const tooltipTriggerList = document.querySelectorAll('[data-bs-toggle="tooltip"]');
      const tooltipList = [...tooltipTriggerList].map(tooltipTriggerEl => new bootstrap.Tooltip(tooltipTriggerEl, {
        html: true,
        sanitize: false,
      }));
      for (let i = 0; i < turns.length; i++) {
        let onInsert = function () {
          let tooltipCanvas = document.getElementById("tooltipCanvas"+i);
          tooltipCanvas.width = CANVAS_SIZE;
          tooltipCanvas.height = CANVAS_SIZE;
          var context = tooltipCanvas.getContext('2d');
          drawHistory(context, i);
        }
        tooltipTriggerList[i].addEventListener("inserted.bs.tooltip", onInsert);
      }

      loadStats();

    }

  }

  draw();

  if (awaitingMoveFrom == "ai") { dispatchAIMove(); }
}

// For debugging
socket.onAny((event, ...args) => {
  console.log(event, args);
});

function getUserCreds() {
  var userID = localStorage.getItem("userID");
  var password = localStorage.getItem("password");
  if (userID == null || password == null) {
    var request = new XMLHttpRequest();
    request.open("GET", "/api/newUser", false);
    request.send();
    let id = JSON.parse(request.responseText).id;
    let password = JSON.parse(request.responseText).password;
    localStorage.setItem("userID", id);
    localStorage.setItem("password", password);
    userID = id + "";
    password = password + "";
  }
  return [userID, password];
}

var charts = [];
window.charts = charts;
function loadStats() {

  for (let chart of charts) {
    chart.destroy();
  }
  charts = [];

  var request = new XMLHttpRequest();
  request.open("GET", "/api/stats/" + getUserCreds()[0] + "/" + dateString, false);
  request.send();
  let stats = JSON.parse(request.responseText);
  
  charts.push(new Chart(document.getElementById('othersPercentageWin'), {
    type: 'doughnut',
    data: {
      labels: ['Wins', 'Draws', 'Losses'],
      datasets: [{
        data: [stats.dNumWins, stats.dNumDraws, stats.dNumLosses],
        backgroundColor: [
          '#6fc276',
          'grey',
          '#e45154'
        ],
        hoverOffset: 10,
      }]
    },
    options: {
      plugins: {
        legend: {
          display: false
        }
      }
    }  
  }));

  charts.push(new Chart(document.getElementById('userPercentageWin'), {
    type: 'doughnut',
    data: {
      labels: ['Wins', 'Draws', 'Losses'],
      datasets: [{
        data: [stats.uNumWins, stats.uNumDraws, stats.uNumLosses],
        backgroundColor: [
          '#6fc276',
          'grey',
          '#e45154'
        ],
        hoverOffset: 10,
      }]
    },
    options: {
      plugins: {
        legend: {
          display: false
        }
      }
    }  
  }));

  let greys = [];
  for (var i = 0; i <= 10; i++) {
    greys.push('rgb(201, 203, 207)');
  }
  let ones = [];
  for (var i = 0; i <= 10; i++) {
    ones.push(1);
  }

  charts.push(new Chart(document.getElementById('dateMovesWin'), {
    type: 'bar',
    data: {
      labels: ["1-9", "10-19", "20-29", "30-39", "40-49", "50-59", "60-69", "70-79", "80-89", "90-99", "100+"],
      datasets: [{
        data: stats.dWinMoves,
        backgroundColor: '#6fc276',
        borderColor: [...greys],
        borderWidth: [...ones],
        barPercentage: 1,
        categoryPercentage: 1,
      }],
    },
    options: {
      scales: {
        y: {
          beginAtZero: true,
          title: {
            display: true,
            text: "Number of wins"
          }
        },
        x: {
          title: {
            display: true,
            text: "Number of moves"
          }
        }
      },
      plugins: {
        legend: {
          display: false
        }
      }
    },
  }));

  if (localStorage.getItem("mostRecentWinner") == "You") {
    let index = Math.min(Math.floor(localStorage.getItem("mostRecentNumMoves") / 10), 10)
    charts[charts.length - 1].data.datasets[0].borderColor[index] = 'rgb(20, 20, 20)';
    charts[charts.length - 1].data.datasets[0].borderWidth[index] = 2;
  }

  charts.push(new Chart(document.getElementById('dateMovesLoss'), {
    type: 'bar',
    data: {
      labels: ["1-9", "10-19", "20-29", "30-39", "40-49", "50-59", "60-69", "70-79", "80-89", "90-99", "100+"],
      datasets: [{
        data: stats.dLossMoves,
        backgroundColor: '#e45154',
        borderColor: [...greys],
        borderWidth: [...ones],
        barPercentage: 1,
        categoryPercentage: 1,
      }]
    },
    options: {
      scales: {
        y: {
          beginAtZero: true,
          title: {
            display: true,
            text: "Number of losses"
          }
        },
        x: {
          title: {
            display: true,
            text: "Number of moves"
          }
        }
      },
      plugins: {
        legend: {
          display: false
        }
      }
    },
  }));

  if (localStorage.getItem("mostRecentWinner") == "The computer") {
    let index = Math.min(Math.floor(localStorage.getItem("mostRecentNumMoves") / 10), 10)
    charts[charts.length - 1].data.datasets[0].borderColor[index] = 'rgb(20, 20, 20)';
    charts[charts.length - 1].data.datasets[0].borderWidth[index] = 2;
  }

  charts.push(new Chart(document.getElementById('userMovesWin'), {
    type: 'bar',
    data: {
      labels: ["1-9", "10-19", "20-29", "30-39", "40-49", "50-59", "60-69", "70-79", "80-89", "90-99", "100+"],
      datasets: [{
        data: stats.uWinMoves,
        backgroundColor: '#6fc276',
        borderColor: [...greys],
        borderWidth: [...ones],
        barPercentage: 1,
        categoryPercentage: 1,
      }]
    },
    options: {
      scales: {
        y: {
          beginAtZero: true,
          title: {
            display: true,
            text: "Number of wins"
          }
        },
        x: {
          title: {
            display: true,
            text: "Number of moves"
          }
        }
      },
      plugins: {
        legend: {
          display: false
        }
      }
    },
  }));

  if (localStorage.getItem("mostRecentWinner") == "You") {
    let index = Math.min(Math.floor(localStorage.getItem("mostRecentNumMoves") / 10), 10)
    charts[charts.length - 1].data.datasets[0].borderColor[index] = 'rgb(20, 20, 20)';
    charts[charts.length - 1].data.datasets[0].borderWidth[index] = 2;
  }

  charts.push(new Chart(document.getElementById('userMovesLoss'), {
    type: 'bar',
    data: {
      labels: ["1-9", "10-19", "20-29", "30-39", "40-49", "50-59", "60-69", "70-79", "80-89", "90-99", "100+"],
      datasets: [{
        data: stats.uLossMoves,
        backgroundColor: '#e45154',
        borderColor: [...greys],
        borderWidth: [...ones],
        barPercentage: 1,
        categoryPercentage: 1,
      }]
    },
    options: {
      scales: {
        y: {
          beginAtZero: true,
          title: {
            display: true,
            text: "Number of losses"
          }
        },
        x: {
          title: {
            display: true,
            text: "Number of moves"
          }
        }
      },
      plugins: {
        legend: {
          display: false
        }
      }
    },
  }));

  if (localStorage.getItem("mostRecentWinner") == "The computer") {
    let index = Math.min(Math.floor(localStorage.getItem("mostRecentNumMoves") / 10), 10)
    charts[charts.length - 1].data.datasets[0].borderColor[index] = 'rgb(20, 20, 20)';
    charts[charts.length - 1].data.datasets[0].borderWidth[index] = 2;
  }
}