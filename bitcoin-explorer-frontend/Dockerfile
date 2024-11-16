# Stage 1: Build
FROM node:latest AS builder
WORKDIR /app

# Copy package files and install dependencies
COPY package.json package-lock.json ./
RUN npm install

# Copy the rest of the application source code
COPY . .

# Build the application (assumes your app has a build script defined in package.json)
RUN npm run build

# Stage 2: Production - Serve with Nginx
FROM nginx:latest AS production
COPY --from=builder /app/build /usr/share/nginx/html

# Expose port 80 for HTTP traffic
EXPOSE 80

# Run Nginx
CMD ["nginx", "-g", "daemon off;"]
