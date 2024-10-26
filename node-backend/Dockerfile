# Use the official Node.js image.
FROM node:18

# Set the working directory.
WORKDIR /usr/src/app

# Copy package.json and package-lock.json to the working directory.
COPY package*.json ./

# Install production dependencies.
RUN npm install

# Copy the rest of the application code.
COPY . .

# Expose the application's port.
EXPOSE 8080

# Command to run the application.
CMD [ "node", "index.js" ]