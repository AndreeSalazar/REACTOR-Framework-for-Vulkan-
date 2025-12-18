#pragma once
#include <memory>
#include <string>
#include <vector>
namespace reactor {
struct AppConfig {
  bool enableValidation;
};
class App {
 public:
  explicit App(const AppConfig& cfg);
  int run();
 private:
  bool validation;
};
}

