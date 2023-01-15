# syntax=docker/dockerfile:1
FROM node:18-alpine

RUN apk add --no-cache make gcc g++ python3

# Create app directory
WORKDIR /usr/src/app

# Install app dependencies
# A wildcard is used to ensure both package.json AND package-lock.json are copied
# where available (npm@5+)
COPY package*.json ./

RUN npm ci
# If you are building your code for production
# RUN npm ci --only=production

# Bundle app source
COPY . .

RUN npm run build:frontend

RUN npm run build:backend

EXPOSE 8080

CMD [ "npm", "start" ]