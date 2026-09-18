#include "lekhani_addon.h"

#include <fcitx-utils/log.h>
#include <fcitx/candidatelist.h>
#include <fcitx/inputpanel.h>
#include <string>

namespace fcitx {

class LekhaniCandidateWord : public CandidateWord {
public:
    LekhaniCandidateWord(Text text, size_t index, LekhaniState *state)
        : CandidateWord(std::move(text)), index_(index), state_(state) {}

    void select(InputContext *ic) const override {
        FCITX_UNUSED(ic);
        if (state_ && state_->engine()) {
            lekhani_engine_select_candidate(state_->engine(), index_);
            state_->updateUI();
        }
    }

private:
    size_t index_;
    LekhaniState *state_;
};

class LekhaniCandidateList : public CommonCandidateList {
public:
    LekhaniCandidateList() {
        setPageSize(5);
        setLabels({ "1", "2", "3", "4", "5" });
    }
};

LekhaniState::LekhaniState(InputContext *ic, LekhaniAddon *addon)
    : ic_(ic), addon_(addon), engine_(lekhani_engine_new()) {}

LekhaniState::~LekhaniState() {
    if (engine_) {
        lekhani_engine_free(engine_);
        engine_ = nullptr;
    }
}

void LekhaniState::reset() {
    if (engine_) {
        lekhani_engine_reset(engine_);
    }
    updateUI();
}

void LekhaniState::updateUI() {
    if (!ic_ || !engine_) {
        return;
    }

    // 1. Commit text if any
    const char *commit = lekhani_engine_get_commit_text(engine_);
    if (commit && commit[0] != '\0') {
        ic_->commitString(commit);
        lekhani_engine_clear_commit_text(engine_);
    }

    // 2. Client Preedit & Aux Preedit
    const char *preedit = lekhani_engine_get_preedit_text(engine_);
    const char *aux = lekhani_engine_get_auxiliary_text(engine_);

    if (preedit && preedit[0] != '\0') {
        Text preeditText(preedit);
        preeditText.setCursor(preeditText.textLength());
        ic_->inputPanel().setClientPreedit(preeditText);
    } else {
        ic_->inputPanel().setClientPreedit(Text());
    }

    if (aux && aux[0] != '\0') {
        ic_->inputPanel().setAuxUp(Text(aux));
    } else {
        ic_->inputPanel().setAuxUp(Text());
    }
    ic_->updatePreedit();

    // 3. Candidate List
    size_t candidateCount = lekhani_engine_get_candidate_count(engine_);
    if (candidateCount > 0) {
        auto candidateList = std::make_unique<LekhaniCandidateList>();
        size_t selectedIdx = lekhani_engine_get_selected_candidate_index(engine_);
        bool isPrediction = lekhani_engine_is_prediction_mode(engine_);
        bool isNavigated = lekhani_engine_is_prediction_navigated(engine_);

        for (size_t i = 0; i < candidateCount; ++i) {
            const char *cand = lekhani_engine_get_candidate_at(engine_, i);
            if (cand) {
                candidateList->append(std::make_unique<LekhaniCandidateWord>(
                    Text(cand), i, this
                ));
            }
        }
        if (isPrediction && !isNavigated) {
            candidateList->setGlobalCursorIndex(-1);
        } else {
            candidateList->setGlobalCursorIndex(static_cast<int>(selectedIdx));
        }
        ic_->inputPanel().setCandidateList(std::move(candidateList));
    } else {
        ic_->inputPanel().setCandidateList(nullptr);
    }

    ic_->updateUserInterface(UserInterfaceComponent::InputPanel);
}

LekhaniAddon::LekhaniAddon(Instance *instance)
    : instance_(instance),
      factory_([this](InputContext &ic) {
          return new LekhaniState(&ic, this);
      }) {
    instance_->inputContextManager().registerProperty("lekhaniState", &factory_);
}

void LekhaniAddon::activate(const InputMethodEntry &entry, InputContextEvent &event) {
    auto *state = event.inputContext()->propertyFor(&factory_);
    if (state && state->engine()) {
        lekhani_engine_reload_config(state->engine());
        std::string_view name = entry.uniqueName();
        if (name.find("Probhat") != std::string_view::npos) {
            lekhani_engine_set_layout(state->engine(), "Probhat");
        } else if (name.find("National") != std::string_view::npos || name.find("Jatiya") != std::string_view::npos) {
            lekhani_engine_set_layout(state->engine(), "National (Jatiya)");
        } else if (name.find("Unijoy") != std::string_view::npos) {
            lekhani_engine_set_layout(state->engine(), "Unijoy");
        } else if (name.find("Borno") != std::string_view::npos) {
            lekhani_engine_set_layout(state->engine(), "Borno");
        } else if (name.find("Avro") != std::string_view::npos) {
            lekhani_engine_set_layout(state->engine(), "Avro Phonetic");
        }
        // If the entry is generic ("lekhani"), lekhani_engine_reload_config already applied active_layout from config.
        state->reset();
    }
}

void LekhaniAddon::deactivate(const InputMethodEntry &entry, InputContextEvent &event) {
    FCITX_UNUSED(entry);
    auto *state = event.inputContext()->propertyFor(&factory_);
    if (state) {
        state->reset();
    }
}

void LekhaniAddon::reset(const InputMethodEntry &entry, InputContextEvent &event) {
    FCITX_UNUSED(entry);
    auto *state = event.inputContext()->propertyFor(&factory_);
    if (state) {
        state->reset();
    }
}

void LekhaniAddon::keyEvent(const InputMethodEntry &entry, KeyEvent &keyEvent) {
    FCITX_UNUSED(entry);
    auto *ic = keyEvent.inputContext();
    auto *state = ic->propertyFor(&factory_);
    if (!state || !state->engine()) {
        return;
    }

    uint32_t sym = keyEvent.rawKey().sym();
    uint32_t code = keyEvent.rawKey().code();
    uint32_t states = static_cast<uint32_t>(keyEvent.rawKey().states());
    bool isRelease = keyEvent.isRelease();

    bool handled = lekhani_engine_process_key(
        state->engine(), sym, code, states, isRelease
    );

    state->updateUI();

    if (handled) {
        keyEvent.filterAndAccept();
    }
}

FCITX_ADDON_FACTORY(LekhaniEngineFactory);

} // namespace fcitx
