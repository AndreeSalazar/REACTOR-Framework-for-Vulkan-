#pragma once
#include <string>
#include <vector>
#include <functional>
#include <cstdint>

namespace reactor {

/**
 * @brief NetworkClient - Cliente de red simple
 * 
 * Uso simple:
 * NetworkClient client;
 * client.connect("127.0.0.1", 8080);
 * client.send("Hello Server!");
 */
class NetworkClient {
public:
    NetworkClient() = default;
    ~NetworkClient();
    
    /**
     * @brief Conectar a servidor
     */
    bool connect(const std::string& host, uint16_t port);
    void disconnect();
    bool isConnected() const { return connected; }
    
    /**
     * @brief Enviar/Recibir datos
     */
    bool send(const std::string& data);
    bool send(const void* data, size_t size);
    std::string receive();
    
    /**
     * @brief Callbacks
     */
    void onReceive(std::function<void(const std::string&)> callback);
    void onDisconnect(std::function<void()> callback);

private:
    bool connected{false};
    std::function<void(const std::string&)> receiveCallback;
    std::function<void()> disconnectCallback;
};

/**
 * @brief NetworkServer - Servidor de red simple
 */
class NetworkServer {
public:
    NetworkServer() = default;
    ~NetworkServer();
    
    /**
     * @brief Iniciar servidor
     */
    bool start(uint16_t port);
    void stop();
    bool isRunning() const { return running; }
    
    /**
     * @brief Broadcast a todos los clientes
     */
    void broadcast(const std::string& data);
    
    /**
     * @brief Callbacks
     */
    void onClientConnect(std::function<void(int clientId)> callback);
    void onClientDisconnect(std::function<void(int clientId)> callback);
    void onReceive(std::function<void(int clientId, const std::string&)> callback);

private:
    bool running{false};
    std::function<void(int)> connectCallback;
    std::function<void(int)> disconnectCallback;
    std::function<void(int, const std::string&)> receiveCallback;
};

} // namespace reactor
