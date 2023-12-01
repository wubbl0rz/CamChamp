# build rust
FROM rust:1.74-bookworm as rust

WORKDIR /build

RUN apt-get update -y
RUN apt-get install -y wget

RUN wget -q -O - https://github.com/BtbN/FFmpeg-Builds/releases/download/autobuild-2023-11-30-12-55/ffmpeg-n6.0.1-linux64-gpl-shared-6.0.tar.xz | tar Jxv
RUN mv ffmpeg-n* ffmpeg

COPY webrtcrust/Cargo* .

RUN mkdir src; touch src/lib.rs
RUN cargo build --release

COPY webrtcrust/src src
RUN cargo build --release

# build dotnet
FROM mcr.microsoft.com/dotnet/sdk:8.0 as dotnet

WORKDIR /build

COPY backend/*.cs .
COPY backend/*.csproj .

RUN dotnet publish -c release

# app
FROM mcr.microsoft.com/dotnet/aspnet:8.0-jammy-chiseled as runtime

WORKDIR /app

COPY --from=rust /build/ffmpeg/lib ffmpeg/lib
COPY --from=rust /build/target/release/libwebrtcrust.so .
COPY --from=dotnet /build/bin/release/net8.0/publish .
COPY backend/wwwroot wwwroot

CMD ["backend.dll", "--urls 'http://*:8044'"]

