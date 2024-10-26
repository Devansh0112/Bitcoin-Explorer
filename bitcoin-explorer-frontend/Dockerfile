FROM node:latest AS builder
WORKDIR /app

COPY package.json /app
COPY package-lock.json /app

RUN npm install
COPY . .
RUN npm run build

FROM nginx:latest AS production
COPY --from=builder /app/build /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]