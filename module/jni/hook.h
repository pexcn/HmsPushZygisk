#ifndef HMSPUSHZYGISK_HOOK_H
#define HMSPUSHZYGISK_HOOK_H


#include <jni.h>
#include <fcntl.h>
#include "zygisk.hpp"

using zygisk::Api;

class Hook {
public:
    Hook(Api *api, JNIEnv *env, bool skipBuild = false) {
        this->api = api;
        this->env = env;
        this->skipBuild = skipBuild;
    }

    void hook();

private:
    Api *api;
    JNIEnv *env;
    bool skipBuild;
};

#endif //HMSPUSHZYGISK_HOOK_H