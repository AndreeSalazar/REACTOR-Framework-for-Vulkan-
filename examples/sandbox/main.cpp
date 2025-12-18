#include "reactor/reactor.hpp"
#include <iostream>
int main() {
  reactor::AppConfig cfg{true};
  reactor::App app(cfg);
  int rc = app.run();
  std::cout << "reactor sandbox " << rc << std::endl;
  return rc;
}

