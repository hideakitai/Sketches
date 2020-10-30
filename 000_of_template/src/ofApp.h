#pragma once

#include "ofMain.h"

class ofApp : public ofBaseApp {

    ofShader shader;
    ofFbo fbo;
    
public:

    void setup() {
        ofSetVerticalSync(false);
        ofSetFrameRate(60);
        ofSetBackgroundColor(0);
        
        shader.load("shader/shader.vert", "shader/shader.frag");
    }

    void update() {
        
    }

    void draw() {
        shader.begin();
        {
            shader.setUniform1f("u_time", ofGetElapsedTimef());
            shader.setUniform2f("u_resolution", ofGetWidth(), ofGetHeight());
            ofDrawRectangle(0, 0, ofGetWidth(), ofGetHeight());
        }
        shader.end();
    }

    void keyPressed(const int key) {
        switch (key) {
            case ' ': {
                break;
            }
            default: {
                break;
            }
        }
    }
};
