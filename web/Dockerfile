# ==============================================================================
# Build Stage - Node.js 22 LTS
# ==============================================================================
FROM node:22-alpine AS builder

WORKDIR /app

# Cache dependencies
COPY package.json package-lock.json* ./
RUN npm install

# Build static assets with Vite and TypeScript
COPY . .
RUN npm run build

# ==============================================================================
# Production Stage - Nginx Alpine
# ==============================================================================
FROM nginx:1.27-alpine

# Copy custom Nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy compiled static assets from builder
COPY --from=builder /app/dist /usr/share/nginx/html

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
