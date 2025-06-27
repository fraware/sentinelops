FROM ubuntu:22.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential git cmake python3 python3-pip wget curl \
    libmodbus-dev liblua5.3-dev libsqlite3-dev libmicrohttpd-dev

# Clone OpenPLC repo
RUN git clone https://github.com/thiagoralves/OpenPLC_v3.git /OpenPLC_v3

# Build OpenPLC
WORKDIR /OpenPLC_v3
RUN chmod +x install.sh && ./install.sh linux

EXPOSE 502

CMD ["python3", "webserver/main.py"]
