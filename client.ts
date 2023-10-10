// const readline = require('readline');

// const socket = new WebSocket('ws://localhost:3000');
// socket.send('Salut gros !');

// // socket.addEventListener("message", (e) => {
// //   const data = JSON.parse(e.data);
// //   if (data.type == "init") {
// //     userName = data.userName;
// //     read();
// //   } else console.log(`${data.userName}:${data.message}`);
// // });

// // socket.addEventListener("open", ({ data }) => {});

// let rl;
// function read(id = 0) {
//   if (rl === undefined)
//     rl = readline.createInterface({
//       terminal: true,
//       input: process.stdin,
//       output: process.stdout,
//     });

//   rl.question('', (message) => {
//     socket.send(message);
//     readline.moveCursor(process.stdout, 0, -1);
//     readline.clearScreenDown(process.stdout);
//     // rl.close();

//     read(id + 1);
//   });
// }

// read();
var net = require('net');
let client = new net.Socket();
client.connect('3000', 'localhost', () => {
  console.log('Connected');
  client.write(Buffer.from(Math.random().toString(), 'utf8')); //This will send the byte buffer over TCP
});
