FROM node:20-alpine
WORKDIR /app
COPY . .
RUN npm ci
EXPOSE 5173
CMD ["npm","run","dev","--","--host","0.0.0.0"]
