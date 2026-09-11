#pragma once

#include <fcitx/addonfactory.h>
#include <fcitx/addoninstance.h>
#include <fcitx/addonmanager.h>
#include <fcitx/candidatelist.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextproperty.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputpanel.h>
#include <fcitx/instance.h>
#include <fcitx/text.h>

#include "lekhani_ffi.h"

namespace fcitx {

class LekhaniAddon;

class LekhaniState : public InputContextProperty {
public:
    LekhaniState(InputContext *ic, LekhaniAddon *addon);
    ~LekhaniState() override;

    void reset();
    void updateUI();

    LekhaniEngineContext *engine() const { return engine_; }
    InputContext *ic() const { return ic_; }

private:
    InputContext *ic_;
    LekhaniAddon *addon_;
    LekhaniEngineContext *engine_;
};

class LekhaniAddon : public InputMethodEngineV2 {
public:
    LekhaniAddon(Instance *instance);
    ~LekhaniAddon() override = default;

    void keyEvent(const InputMethodEntry &entry, KeyEvent &keyEvent) override;
    void activate(const InputMethodEntry &entry, InputContextEvent &event) override;
    void deactivate(const InputMethodEntry &entry, InputContextEvent &event) override;
    void reset(const InputMethodEntry &entry, InputContextEvent &event) override;

    Instance *instance() const { return instance_; }
    FactoryFor<LekhaniState> &factory() { return factory_; }

private:
    Instance *instance_;
    FactoryFor<LekhaniState> factory_;
};

class LekhaniEngineFactory : public AddonFactory {
public:
    AddonInstance *create(AddonManager *manager) override {
        return new LekhaniAddon(manager->instance());
    }
};

} // namespace fcitx
