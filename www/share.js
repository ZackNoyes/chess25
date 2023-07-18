import { drawHistory } from "./utils";
import { CANVAS_SIZE } from "./constants";

var games = [];
var currentMove = 1;

let b64Game = new URLSearchParams(window.location.search).get('game');
let gameString = atob(b64Game);
let split = gameString.split(':');

let id = split[0];
let dateString = split[1];
let dateArr = dateString.split('_');
let year = +dateArr[0];
let month = +dateArr[1];
let day = +dateArr[2];
let gameDate = new Date(year, month - 1, day);

for (let elem of document.getElementsByClassName('dateOfGame')) {
  elem.innerHTML = gameDate.toLocaleDateString(undefined, {
    year: "numeric", month: 'short', day: 'numeric'
  });
}

let now = new Date();
let currentDate = new Date(now.getFullYear(), now.getMonth() - 1, now.getDate());

var myGame = null;
var theirGame = null;
var showGames = false;

var request = new XMLHttpRequest();
request.open("GET", "/api/game/" + b64Game, false);
request.send();

if (request.status == 200) {
  theirGame = JSON.parse(request.responseText);

  if (gameDate.getTime() > currentDate.getTime()) {
    document.getElementById('futureDay').style.display = 'block';
  } else if (gameDate.getTime() == currentDate.getTime()) {

    let myID = localStorage.getItem('userID');

    if (myID == null) {
      document.getElementById('notPlayedToday').style.display = 'block';
    } else if (myID == id) {
      myGame = theirGame;
      theirGame = null;
      showGames = true;
    } else {

      request = new XMLHttpRequest();
      request.open("GET", "/api/game/" + btoa(myID + ":" + dateString), false);
      request.send();

      if (request.status == 200) {
        myGame = JSON.parse(request.responseText);
        showGames = true;
      } else if (request.status == 204) {
        document.getElementById('notPlayedToday').style.display = 'block';
      } else {
        console.error("Unexpected status code: " + request.status);
      }

    }

  } else {
    showGames = true;

    let myID = localStorage.getItem('userID');
    if (myID == id) {
      myGame = theirGame;
      theirGame = null;
    } else if (myID != null) {
      request = new XMLHttpRequest();
      request.open("GET", "/api/game/" + btoa(myID + ":" + dateString), false);
      request.send();
      if (request.status == 200) {
        myGame = JSON.parse(request.responseText);
      }
    }

  }

} else if (request.status == 204) {
  document.getElementById('gameNotFound').style.display = 'block';
} else {
  console.error("Unexpected status code: " + request.status);
}

if (showGames) {

  document.getElementById('gamesDisplay').style.display = 'block';

  if (myGame != null && theirGame != null) {
    
    document.getElementById('gameComparison').style.display = 'block';
    document.getElementById('ownResult').innerHTML = myGame.resultString;
    document.getElementById('theirResult').innerHTML = theirGame.resultString;

    addCanvas(document.getElementById('ownGameCol'), myGame);
    addCanvas(document.getElementById('theirGameCol'), theirGame);

  } else if (myGame != null) {

    document.getElementById('singleGame').style.display = 'block';
    document.getElementById('ownGameExplanation').style.display = 'block';
    document.getElementById('singleResult').innerHTML = myGame.resultString;

    addCanvas(document.getElementById('singleGameCol'), myGame);

  } else if (theirGame != null) {

    document.getElementById('singleGame').style.display = 'block';
    document.getElementById('onlyTheirGameExplanation').style.display = 'block';
    document.getElementById('singleResult').innerHTML = theirGame.resultString;

    addCanvas(document.getElementById('singleGameCol'), theirGame);

  }

  if (myGame != null) {
    document.getElementById('shareButton').style.display = 'inline';
  }

}

function addCanvas(colElem, game) {
  let canvas = document.createElement('canvas');
  canvas.classList.add('static-canvas');
  canvas.width = CANVAS_SIZE;
  canvas.height = CANVAS_SIZE;
  colElem.appendChild(canvas);
  var context = canvas.getContext('2d');

  let turnLabel = document.createElement('p');
  colElem.appendChild(turnLabel);

  games.push({context: context, game: game, turnLabel: turnLabel});
  draw();
}

function incMove() {
  currentMove++;
  var overshot = true;
  for (let game of games) {
    if (game.game.turns.length >= currentMove) {
      overshot = false;
      break;
    }
  }
  if (overshot) { currentMove--; }
  draw();
}
window.incMove = incMove;

function decMove() {
  currentMove--;
  if (currentMove < 1) { currentMove = 1; }
  draw();
}
window.decMove = decMove;

function draw() {

  var longestTurns = []

  for (let game of games) {
    let move = Math.min(currentMove, game.game.turns.length);
    drawHistory(game.context, move - 1, game.game.history);
    game.turnLabel.innerHTML = "move " + move;
    if (game.game.turns.length > longestTurns.length) {
      longestTurns = game.game.turns;
    }
  }

  document.getElementById("moveHistory").innerHTML = "";
  for (let i = 0; i < longestTurns.length; i++) {
    let box = document.createElement('button');
    box.classList.add(longestTurns[i] == "w" ? "white-box" : "black-box");
    if (i + 1 == currentMove) {
      box.classList.add("current-move");
    }
    for (let game of games) {
      if (game.game.turns.length == i + 1) {
        box.classList.add("final-move");
      }
    }
    box.onclick = function() {
      currentMove = i + 1;
      draw();
    };
    document.getElementById("moveHistory").appendChild(box);
  }

  document.getElementById("decButton").disabled = currentMove == 1;
  document.getElementById("incButton").disabled = currentMove == longestTurns.length;

}

document.onkeydown = function(e) {
  if (e.key == "ArrowLeft") {
    decMove();
  }
  if (e.key == "ArrowRight") {
    incMove();
  }
}

function shareGame() {
  let button = document.getElementById("shareButton");
  navigator.clipboard.writeText(window.location.hostname + "/share/?game=" + btoa(localStorage.getItem("userID") + ":" + dateString)).then(() => {
    var tooltip = new bootstrap.Tooltip(button, {title: "Link copied to clipboard"});
    tooltip.show();
    setTimeout(() => {
      tooltip.hide();
    }, 2000);
  },
  () => {
    var tooltip = new bootstrap.Tooltip(button, {title: "Link copied to clipboard"});
    tooltip.show();
    setTimeout(() => {
      tooltip.hide();
    }, 2000);
  });
}
window.shareGame = shareGame;