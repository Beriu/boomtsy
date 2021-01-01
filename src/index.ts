import App from "./app/App";

const app = new App();

app.start()
    .then(() => console.info('Bootstrap finished.'))
    .catch((error) => console.error(error));