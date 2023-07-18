import {
  SQUARE_SIZE,
  BLACK_SQUARE_COLOR,
  WHITE_SQUARE_COLOR,
  ACTIVE_COLOR,
  PIECES
} from "./constants.js";

function flip(rank) {
  return 7 - rank;
}

export async function drawHistory(context, i, history) {
  await imagePromise;
  for (let file = 0; file < 8; file++) {
    for (let rank = 0; rank < 8; rank++) {
      if ((file + flip(rank)) % 2 == 0) {
        context.fillStyle = BLACK_SQUARE_COLOR;
      } else {
        context.fillStyle = WHITE_SQUARE_COLOR;
      }
      context.fillRect(file * SQUARE_SIZE, rank * SQUARE_SIZE, SQUARE_SIZE, SQUARE_SIZE);
      if (history[i][file][flip(rank)][1]) {
        context.fillStyle = ACTIVE_COLOR;
        context.fillRect(file * SQUARE_SIZE, rank * SQUARE_SIZE, SQUARE_SIZE, SQUARE_SIZE);
      }
      var piece = history[i][file][flip(rank)][0];
      if (piece != undefined) {
        let img = await getImage(piece);
        context.drawImage(img, file * SQUARE_SIZE, rank * SQUARE_SIZE, SQUARE_SIZE, SQUARE_SIZE);
      }
    }
  }
}

export async function getImage(piece) {
  await imagePromise;
  return images[piece];
}

const images = {};

const loadImage = piece =>
  new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => resolve(img);
    img.onerror = reject;
    var t = piece.toUpperCase();
    var c = t == piece ? 'w' : 'b';
    img.src = '/images/cburnett/' + c + t + '.svg';
    images[piece] = img;
  });

var imagePromise = Promise.all(PIECES.map(loadImage));