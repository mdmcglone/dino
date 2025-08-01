CXX = g++
CXXFLAGS = -std=c++17 -Wall -O2 -I./include
LDFLAGS = -lSDL2 -lSDL2_ttf

# macOS specific flags
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    CXXFLAGS += -I/usr/local/include/SDL2 -I/opt/homebrew/include/SDL2
    LDFLAGS += -L/usr/local/lib -L/opt/homebrew/lib
endif

# Linux specific flags
ifeq ($(UNAME_S),Linux)
    CXXFLAGS += $(shell sdl2-config --cflags)
    LDFLAGS += $(shell sdl2-config --libs)
endif

TARGET = neon_pangaea
SRCDIR = src
OBJDIR = obj

SOURCES = $(wildcard $(SRCDIR)/*.cpp)
OBJECTS = $(patsubst $(SRCDIR)/%.cpp,$(OBJDIR)/%.o,$(SOURCES))

all: $(TARGET)

$(TARGET): $(OBJECTS)
	$(CXX) $(OBJECTS) -o $@ $(LDFLAGS)

$(OBJDIR)/%.o: $(SRCDIR)/%.cpp | $(OBJDIR)
	$(CXX) $(CXXFLAGS) -c $< -o $@

$(OBJDIR):
	mkdir -p $(OBJDIR)

clean:
	rm -rf $(OBJDIR) $(TARGET)

run: $(TARGET)
	./$(TARGET)

.PHONY: all clean run 