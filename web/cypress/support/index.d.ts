declare namespace Cypress {
  interface Chainable {
    loginToBuster(email?: string, password?: string): Chainable<void>;
    askQuestion(question: string): Chainable<void>;
  }
}
