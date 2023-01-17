import express from "express";
import { Server } from "socket.io";
import http from "http";
import path from "path";

export default async function runServer() {
    const app = express();
    const server = http.createServer(app);
    const socket = new Server(server);

    app.use('', express.static(path.join(process.env.PWD as string, '/distApp')));

    app.get('/', function(req, res) {
        res.sendFile(path.join(process.env.PWD as string, '/distApp/index.html'));
    });

    socket.on('connection', (socket) => {
        socket.emit("sessions/get", 'Hi there! This is on CibzPi');
    });

    server.listen(process.env.WEBSERVER_PORT, () => {
        console.log(`Webserver running at http://localhost:${process.env.WEBSERVER_PORT}`);
    });
}