#include "reactor/extras/networking.hpp"
#include <iostream>

namespace reactor {

NetworkClient::~NetworkClient() {
    if (connected) {
        disconnect();
    }
}

bool NetworkClient::connect(const std::string& host, uint16_t port) {
    std::cout << "[NetworkClient] Connecting to " << host << ":" << port << std::endl;
    // TODO: Implement actual socket connection
    connected = true;
    return true;
}

void NetworkClient::disconnect() {
    if (!connected) return;
    std::cout << "[NetworkClient] Disconnecting..." << std::endl;
    connected = false;
    if (disconnectCallback) {
        disconnectCallback();
    }
}

bool NetworkClient::send(const std::string& data) {
    if (!connected) return false;
    std::cout << "[NetworkClient] Sending: " << data << std::endl;
    return true;
}

bool NetworkClient::send(const void* data, size_t size) {
    if (!connected) return false;
    std::cout << "[NetworkClient] Sending " << size << " bytes" << std::endl;
    return true;
}

std::string NetworkClient::receive() {
    if (!connected) return "";
    // TODO: Implement actual receive
    return "";
}

void NetworkClient::onReceive(std::function<void(const std::string&)> callback) {
    receiveCallback = callback;
}

void NetworkClient::onDisconnect(std::function<void()> callback) {
    disconnectCallback = callback;
}

NetworkServer::~NetworkServer() {
    if (running) {
        stop();
    }
}

bool NetworkServer::start(uint16_t port) {
    std::cout << "[NetworkServer] Starting on port " << port << std::endl;
    // TODO: Implement actual server
    running = true;
    return true;
}

void NetworkServer::stop() {
    if (!running) return;
    std::cout << "[NetworkServer] Stopping..." << std::endl;
    running = false;
}

void NetworkServer::broadcast(const std::string& data) {
    if (!running) return;
    std::cout << "[NetworkServer] Broadcasting: " << data << std::endl;
}

void NetworkServer::onClientConnect(std::function<void(int)> callback) {
    connectCallback = callback;
}

void NetworkServer::onClientDisconnect(std::function<void(int)> callback) {
    disconnectCallback = callback;
}

void NetworkServer::onReceive(std::function<void(int, const std::string&)> callback) {
    receiveCallback = callback;
}

} // namespace reactor
