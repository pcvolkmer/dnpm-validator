#include "mainwindow.h"
#include "ui_mainwindow.h"

#include <QDebug>

MainWindow::MainWindow(QWidget *parent) : QMainWindow(parent), ui(new Ui::MainWindow) {
    ui->setupUi(this);
    this->onLineNumbersChanged(1);

    QFont monospaceFont("monospace");
    monospaceFont.setStyleHint(QFont::TypeWriter);

    this->ui->lineNumbers->setFont(monospaceFont);
    this->ui->plainTextEdit->setFont(monospaceFont);
    this->ui->errorListWidget->setFont(monospaceFont);

    this->positionLabel = new QLabel(this);
    this->formatSelection = new QComboBox(this);
    this->formatSelection->addItem("DNPM Datenmodell 2.1");
    this->formatSelection->addItem("SE:dip Datenmodell");
    this->formatSelection->addItem("GRZ Metadata 1.3.1");
    this->ui->toolBar->addWidget(this->formatSelection);
    this->ui->statusbar->addPermanentWidget(this->positionLabel);

    connect(ui->actionOpen, &QAction::triggered, this, &MainWindow::onOpenAction);
    connect(ui->actionSave, &QAction::triggered, this, &MainWindow::onSaveAction);
    connect(ui->actionSaveAs, &QAction::triggered, this, &MainWindow::onSaveAsAction);
    connect(ui->actionValidate, &QAction::triggered, this, &MainWindow::onValidateAction);
    connect(this->formatSelection, &QComboBox::currentTextChanged, [this](const QString &) {
        this->onValidateAction();
    });
    connect(ui->actionAbout, &QAction::triggered, [this] {
        QMessageBox::about(this,
                           "About DNPM-Validator",
                           R"(
<html><body>
<p style="font-size: large; font-weight: bold;">DNPM-Validator</p>
<p>Application to validate and edit a data set in DNPM Datenmodell 2.1, SE:dip data model and GRZ Metadata format</p>
<p><a href="https://github.com/pcvolkmer/dnpm-validator">https://github.com/pcvolkmer/dnpm-validator</a></p>
</body></html>)"
        );
    });

    connect(ui->plainTextEdit, &QPlainTextEdit::blockCountChanged, this, &MainWindow::onLineNumbersChanged);
    connect(ui->plainTextEdit->verticalScrollBar(), &QScrollBar::valueChanged, [this](const int value) {
        ui->lineNumbers->verticalScrollBar()->setValue(value);
    });

    connect(ui->errorListWidget, &QListWidget::itemClicked,
            [this](const QListWidgetItem *item) {
                const auto row = item->listWidget()->currentIndex().row();
                this->onErrorSelected(row);
            });
}

MainWindow::~MainWindow() {
    delete this->formatSelection;
    delete this->positionLabel;
    delete ui;
}

void MainWindow::onOpenAction() {
    this->filename = QFileDialog::getOpenFileName(
        this,
        "Open file",
        QDir::homePath(),
        "JSON files (*.json);;All files (*.*)"
    );

    if (!this->filename.isEmpty()) {
        if (QFile file(this->filename); file.open(QIODevice::ReadOnly | QIODevice::Text)) {
            const QByteArray content = file.readAll();

            ui->plainTextEdit->setPlainText(
                QString::fromUtf8(content)
            );

            this->onValidateAction();
        }
        this->setWindowTitle(QString("DNPM-Validator :: %1").arg(QFileInfo(this->filename).fileName()));
        return;
    }
    this->setWindowTitle("DNPM-Validator");
}

void MainWindow::onSaveAction() {
    if (!this->filename.isEmpty()) {
        if (QFile file(this->filename); file.open(QIODevice::WriteOnly | QIODevice::Text)) {
            this->onValidateAction();
            file.write(this->ui->plainTextEdit->toPlainText().toUtf8());
            file.close();
        }
        return;
    }
    this->onSaveAsAction();
}

void MainWindow::onSaveAsAction() {
    const auto selectedFilename = QFileDialog::getSaveFileName(
        this,
        "Save file",
        QDir::homePath(),
        "JSON files (*.json);;All files (*.*)"
    );
    if (!selectedFilename.isEmpty()) {
        this->filename = selectedFilename;
        this->onSaveAction();
        this->setWindowTitle(QString("DNPM-Validator :: %1").arg(QFileInfo(this->filename).fileName()));
        return;
    }
    this->setWindowTitle("DNPM-Validator");
}

void MainWindow::onValidateAction() {
    const auto json = ui->plainTextEdit->toPlainText();
    auto validationType = dnpmvalidation::ValidationType::Mtb;
    if (this->formatSelection->currentIndex() == 1) {
        validationType = dnpmvalidation::ValidationType::Rd;
    } else if (this->formatSelection->currentIndex() == 2) {
        validationType = dnpmvalidation::ValidationType::Grz;
    }
    auto errors = dnpmvalidation::validate(rust::String(json.toStdString()), validationType);

    this->errorList.clear();
    this->ui->errorListWidget->clear();

    if (errors.empty()) {
        this->markErrors();
        return;
    }

    for (auto error: errors) {
        this->errorList.push_back(error);
        ui->errorListWidget->addItem(
            QString("[%1:%2]   %3").arg(error.startLine, 4).arg(error.startColumn, 4).arg(error.message.c_str()));
    }
    this->markErrors();
}

void MainWindow::onErrorSelected(const int index) const {
    const auto error = this->errorList.at(index);
    auto cursor = ui->plainTextEdit->textCursor();

    if (error.startLine == 0 || error.startColumn == 0) {
        return;
    }

    ui->plainTextEdit->setFocus();

    cursor.clearSelection();
    cursor.setPosition(
        QTextCursor::Start
    );

    auto startTextBlock = ui->plainTextEdit->document()->findBlockByLineNumber(error.startLine - 1);
    cursor.setPosition(startTextBlock.position() + error.startColumn - 1, QTextCursor::MoveAnchor);
    ui->plainTextEdit->setTextCursor(cursor);
    ui->plainTextEdit->ensureCursorVisible();

    ui->lineNumbers->verticalScrollBar()->setValue(ui->plainTextEdit->verticalScrollBar()->value());
}

void MainWindow::onLineNumbersChanged(int lineCount) const {
    QStringList lines;
    for (int i = 1; i <= lineCount; i++) {
        lines += QString::number(i).rightJustified(6, ' ');
    }
    this->ui->lineNumbers->setPlainText(lines.join('\n'));
}

void MainWindow::markErrors() {
    auto cursor = ui->plainTextEdit->textCursor();

    QList<QTextEdit::ExtraSelection> lineExtraSelections;
    QList<QTextEdit::ExtraSelection> textExtraSelections;

    for (const auto &error: this->errorList) {
        if (error.startLine == 0 || error.startColumn == 0) {
            continue;
        }

        QTextEdit::ExtraSelection lineSelection;
        lineSelection.cursor = QTextCursor(ui->lineNumbers->document()->findBlockByLineNumber(error.startLine - 1));
        QTextCharFormat lineFormat;
        lineFormat.setForeground(Qt::red);
        lineFormat.setFontWeight(QFont::Bold);
        lineFormat.setProperty(QTextFormat::FullWidthSelection, true);
        lineSelection.format = lineFormat;
        lineExtraSelections.append(lineSelection);

        auto startTextBlock = ui->plainTextEdit->document()->findBlockByLineNumber(error.startLine - 1);
        cursor.setPosition(startTextBlock.position() + error.startColumn - 1, QTextCursor::MoveAnchor);

        auto endTextBlock = ui->plainTextEdit->document()->findBlockByLineNumber(error.endLine - 1);
        cursor.setPosition(endTextBlock.position() + error.endColumn - 1, QTextCursor::KeepAnchor);

        QTextEdit::ExtraSelection textSelection;
        textSelection.cursor = cursor;
        QTextCharFormat textFormat;
        textFormat.setUnderlineStyle(QTextCharFormat::WaveUnderline);
        textFormat.setUnderlineColor(Qt::red);
        textSelection.format = textFormat;
        textExtraSelections.append(textSelection);
    }

    ui->lineNumbers->setExtraSelections(lineExtraSelections);
    ui->plainTextEdit->setExtraSelections(textExtraSelections);
}
