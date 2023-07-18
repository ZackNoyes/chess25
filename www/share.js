import { drawHistory } from "./utils";
import { CANVAS_SIZE } from "./constants";

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

// TODO: border coloring

if (showGames) {

  if (myGame != null && theirGame != null) {
    
    document.getElementById('gameComparison').style.display = 'block';
    document.getElementById('ownResult').innerHTML = myGame.resultString;
    document.getElementById('theirResult').innerHTML = theirGame.resultString;

    fillCol(document.getElementById('ownGameCol'), myGame);
    fillCol(document.getElementById('theirGameCol'), theirGame);

  } else if (myGame != null) {

    document.getElementById('singleGame').style.display = 'block';
    document.getElementById('ownGameExplanation').style.display = 'block';
    document.getElementById('singleResult').innerHTML = myGame.resultString;

    fillCol(document.getElementById('singleGameCol'), myGame);

  } else if (theirGame != null) {

    document.getElementById('singleGame').style.display = 'block';
    document.getElementById('onlyTheirGameExplanation').style.display = 'block';
    document.getElementById('singleResult').innerHTML = theirGame.resultString;

    fillCol(document.getElementById('singleGameCol'), theirGame);

  }

}

function fillCol(colElem, game) {
  for (let i = 0; i < game.turns.length; i++) {
    let canvas = document.createElement('canvas');
    canvas.classList.add('static-canvas');
    canvas.width = CANVAS_SIZE;
    canvas.height = CANVAS_SIZE;
    colElem.appendChild(canvas);
    var context = canvas.getContext('2d');
    drawHistory(context, i, game.history);
  }
}