FROM nginx:1.27-alpine
# Đóng gói cấu hình vào image (không bind-mount), để chạy được ở mọi môi trường, kể cả đường dẫn có dấu.
COPY nginx.conf /etc/nginx/nginx.conf
RUN nginx -t
